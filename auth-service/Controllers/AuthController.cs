using System.IdentityModel.Tokens.Jwt;
using System.Security.Claims;
using Google.Apis.Auth;
using Microsoft.AspNetCore.Mvc;
using Microsoft.EntityFrameworkCore;
using BC = BCrypt.Net.BCrypt;

[ApiController]
[Route("[controller]")]
public class AuthController : ControllerBase
{
    private readonly PostgresDbContext _dbContext;
    private readonly ILogger<AuthController> _logger;
    private readonly TokenService _tokenService;
    private readonly CookieService _cookieService;
    private readonly string _client_id = "355255212720-fcovem0bl4uo6au8qpcmc6f6kjbs6mhv.apps.googleusercontent.com";

    public class GoogleSignInRequest
    {
        public required string Credential { get; set; }
    }

    public AuthController(PostgresDbContext dbContext, TokenService tokenService, CookieService cookieService, ILogger<AuthController> logger)
    {
        _dbContext = dbContext;
        _tokenService = tokenService;
        _cookieService = cookieService;
        _logger = logger;
    }


    [HttpPost("signin")]
    public async Task<IActionResult> SignIn([FromBody] UserRegistrationDto user)
    {
        if (user == null)
        {
            _logger.LogWarning("An unexpected result occurred while deserializing request body");
            return BadRequest("Invalid request body");
        }

        try
        {
            var newUser = new User { Email = user.Email, Password = BC.HashPassword(user.Password), Name = user.Password, CountryCode = user.CountryCode };

            await _dbContext.Users.AddAsync(newUser);
            await _dbContext.SaveChangesAsync();
            return Created("/signin", "User created successfully");
        }
        catch (Exception err)
        {
            _logger.LogError(err, "Error saving new user in database");
            return StatusCode(500, "Internal server error");
        }
    }

    [HttpPost("signin-google")]
    public async Task<IActionResult> SignInByGoogle([FromBody] GoogleSignInRequest request)
    {
        GoogleJsonWebSignature.Payload payload;

        try
        {
            payload = await GoogleJsonWebSignature.ValidateAsync(request.Credential, new GoogleJsonWebSignature.ValidationSettings
            {
                Audience = [_client_id]
            });
        }
        catch
        {
            return Unauthorized();
        }


        var user = await _dbContext.Users.FirstOrDefaultAsync(u => u.Email == payload.Email);

        if (user != null)
        {
            if (user.GoogleId != payload.Subject)
            {
                user.GoogleId = payload.Subject;
                _dbContext.Users.Update(user);
                await _dbContext.SaveChangesAsync();
            }
        }
        else
        {
            User newUser = new User
            {
                Email = payload.Email,
                Name = payload.Name,
                GoogleId = payload.Subject,
            };

            await _dbContext.Users.AddAsync(newUser);
            await _dbContext.SaveChangesAsync();
            user = newUser;
        };
        var token = _tokenService.GenerateToken(user);
        var refreshTokenEntity = _tokenService.GenerateRefreshTokenEntity(user.Id);

        await _dbContext.RefreshTokens.AddAsync(refreshTokenEntity);

        await _dbContext.SaveChangesAsync();

        _cookieService.SetCookie(Response, "access_token", token, SameSiteMode.Strict, true, true, TimeSpan.FromHours(3));
        _cookieService.SetCookie(Response, "refresh_token", refreshTokenEntity.Token, SameSiteMode.Strict, true, true, TimeSpan.FromDays(7), "refresh_token");

        var AccessTokenExpireTime = ((DateTimeOffset)DateTime.UtcNow.AddHours(3)).ToUnixTimeSeconds();
        var RefreshTokenExpireTime = ((DateTimeOffset)DateTime.UtcNow.AddDays(7)).ToUnixTimeSeconds();

        return Ok(new { User = new UserDto(user), AccessTokenExpireTime, RefreshTokenExpireTime });
    }


    [HttpGet("refresh-token")]
    public async Task<IActionResult> RefreshAccessToken()
    {
        string? refreshToken;

        var refreshTokenExists = Request.Cookies.TryGetValue("refresh_token", out refreshToken);
        if (!refreshTokenExists || refreshToken == null)
        {
            return Unauthorized();
        }

        var tokens = _dbContext.RefreshTokens.Count();

        var dbRefreshToken = await _dbContext.RefreshTokens.FirstOrDefaultAsync(t => t.Token == refreshToken);

        if (dbRefreshToken != null && dbRefreshToken.IsActive)
        {
            var user = await _dbContext.Users.FirstOrDefaultAsync(u => u.Id == dbRefreshToken.UserId);

            if (user == null)
            {
                return Unauthorized();
            }
            var token = _tokenService.GenerateToken(user);
            var refreshTokenEntity = _tokenService.GenerateRefreshTokenEntity(user.Id);

            await _dbContext.RefreshTokens.AddAsync(refreshTokenEntity);
            await _dbContext.SaveChangesAsync();

            _cookieService.SetCookie(Response, "access_token", token, SameSiteMode.Strict, true, true, TimeSpan.FromHours(3));
            _cookieService.SetCookie(Response, "refresh_token", refreshTokenEntity.Token, SameSiteMode.Strict, true, true, TimeSpan.FromDays(7), "refresh_token");

            var AccessTokenExpireTime = ((DateTimeOffset)DateTime.UtcNow.AddHours(3)).ToUnixTimeSeconds();
            var RefreshTokenExpireTime = ((DateTimeOffset)DateTime.UtcNow.AddDays(7)).ToUnixTimeSeconds();

            return Ok(new { User = new UserDto(user), AccessTokenExpireTime, RefreshTokenExpireTime });
        }
        else
        {
            return Unauthorized();
        }
    }

    [HttpPost("login")]
    public async Task<IActionResult> Login([FromBody] UserRegistrationDto user)
    {
        var userFromDb = await _dbContext.Users.FirstOrDefaultAsync(u => u.Email == user.Email);

        if (userFromDb?.Password != null && BC.Verify(user.Password, userFromDb.Password))
        {
            var token = _tokenService.GenerateToken(userFromDb);
            var refreshTokenEntity = _tokenService.GenerateRefreshTokenEntity(userFromDb.Id);

            await _dbContext.RefreshTokens.AddAsync(refreshTokenEntity);
            await _dbContext.SaveChangesAsync();

            _cookieService.SetCookie(Response, "access_token", token, SameSiteMode.Strict, true, true, TimeSpan.FromHours(3));
            _cookieService.SetCookie(Response, "refresh_token", refreshTokenEntity.Token, SameSiteMode.Strict, true, true, TimeSpan.FromDays(7), "refresh_token");

            var AccessTokenExpireTime = ((DateTimeOffset)DateTime.UtcNow.AddHours(3)).ToUnixTimeSeconds();
            var RefreshTokenExpireTime = ((DateTimeOffset)DateTime.UtcNow.AddDays(7)).ToUnixTimeSeconds();

            return Ok(new { User = new UserDto(userFromDb), AccessTokenExpireTime, RefreshTokenExpireTime });
        }
        else
        {
            return Unauthorized(new { Message = "Invalid credentials" });
        }
    }


    [HttpGet("get-user")]
    public async Task<IActionResult> getUser()
    {
        string? accessToken;

        var accessTokenExists = Request.Cookies.TryGetValue("access_token", out accessToken);

        if (!accessTokenExists || accessToken == null)
        {
            return Unauthorized("Access token not found or invalid.");
        }

        var tokenValidationResult = await _tokenService.ValidateTokenAsync(accessToken);

        if (!tokenValidationResult.IsValid)
        {
            return Unauthorized("Access token validation failed.");
        }

        var userId = tokenValidationResult.ClaimsIdentity.FindFirst(JwtRegisteredClaimNames.Sub);

        if (userId == null)
        {
            return Unauthorized("User ID not found in token claims.");
        }

        var user = await _dbContext.Users
        .Where(u => u.Id.ToString() == userId.Value)
        .FirstOrDefaultAsync();

        if (user == null)
        {
            return NotFound("User not found.");
        }
        else
        {
            return Ok(new { User = new UserDto(user) });
        }

    }

    [HttpPost("unauthorized_session_token")]
    public IActionResult UnauthorizedSessionToken([FromBody] TempUserDto user)
    {
        string? existingToken = Request.Cookies["access_token"];

        if (existingToken != null)
        {
            return BadRequest("Invalid Payload");
        }

        Random random = new Random();

        var id = random.Next(int.MinValue, -1);

        IEnumerable<Claim> claims = [
                new Claim(ClaimTypes.Anonymous, "true"),
                new Claim(JwtRegisteredClaimNames.Sub, id.ToString()),
                new Claim(ClaimTypes.Country, user.CountryCode),
                new Claim(ClaimTypes.Name, user.Name),
        ];
        var token = _tokenService.GenerateUnauthorizedToken(claims);

        Response.Cookies.Append("access_token", token, new CookieOptions
        {
            SameSite = SameSiteMode.Strict,
            HttpOnly = true,
            Secure = true,
            MaxAge = TimeSpan.FromDays(1)
        });

        return Ok(new { Message = "Ok" });
    }


    [HttpGet("logout")]
    public IActionResult Logout()
    {
        // TODO: remove refresh_token from db
        _cookieService.SetCookie(Response, "access_token", "logout", SameSiteMode.Strict, true, true, TimeSpan.FromSeconds(-1));
        _cookieService.SetCookie(Response, "refresh_token", "logout", SameSiteMode.Strict, true, true, TimeSpan.FromSeconds(-1), "refresh_token");

        return Ok();
    }
}