using System.Security.Claims;
using Microsoft.AspNetCore.Mvc;

[ApiController]
[Route("[controller]")]
public class SessionController : ControllerBase
{
    private readonly PostgresDbContext _dbContext;
    private readonly ILogger<AuthController> _logger;
    private readonly TokenService _tokenService;

    public SessionController(PostgresDbContext dbContext, TokenService tokenService, ILogger<AuthController> logger)
    {
        _dbContext = dbContext;
        _tokenService = tokenService;
        _logger = logger;
    }

    public class UserDto
    {
        public required string UserName { get; set; }
        public required string CountryCode { get; set; }
    }

    [HttpPost("unauthorized_session_token")]
    public IActionResult UnauthorizedSessionToken([FromBody] UserDto user)
    {
        string? existingToken = Request.Cookies["auth_token"];

        if (existingToken != null)
        {
           return BadRequest("Invalid Payload");
        }

        Random random = new Random();

        var id = random.Next(int.MinValue, -1);

        IEnumerable<Claim> claims = [
             new Claim(ClaimTypes.Anonymous, "true"),
                new Claim(ClaimTypes.NameIdentifier, id.ToString()),
                new Claim(ClaimTypes.Country, user.CountryCode),
                new Claim(ClaimTypes.Name, user.UserName),
        ];
        var token = _tokenService.GenerateUnauthorizedToken(claims);

        Response.Cookies.Append("auth_token", token, new CookieOptions
        {
            SameSite = SameSiteMode.None,
            HttpOnly = true,
            Secure = true,
            MaxAge = TimeSpan.FromDays(1)
        });

        return Ok(new { Message = "Ok" });
    }
}