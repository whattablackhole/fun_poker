using System.Text.Json;
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

    public AuthController(PostgresDbContext dbContext, TokenService tokenService, ILogger<AuthController> logger)
    {
        _dbContext = dbContext;
        _tokenService = tokenService;
        _logger = logger;
    }

    [HttpPost("signin")]
    public async Task<IActionResult> SignIn([FromBody] User user)
    {
        if (user == null)
        {
            _logger.LogWarning("An unexpected result occurred while deserializing request body");
            return BadRequest("Invalid request body");
        }

        try
        {
            user.Password = BC.HashPassword(user.Password);
            await _dbContext.Users.AddAsync(user);
            await _dbContext.SaveChangesAsync();
            return Created("/signin", "User created successfully");
        }
        catch (Exception err)
        {
            _logger.LogError(err, "Error saving new user in database");
            return StatusCode(500, "Internal server error");
        }
    }

    [HttpPost("login")]
    public async Task<IActionResult> Login([FromBody] User user)
    {
        var userFromDb = await _dbContext.Users.FirstOrDefaultAsync(u => u.Email == user.Email);


        if (userFromDb != null && BC.Verify(user.Password, userFromDb.Password))
        {
            var token = _tokenService.GenerateToken(userFromDb);
            Response.Headers.Append("Authorization", token);
            return Ok(new { Message = "Login successful" });

        }
        else
        {
            return Unauthorized(new { Message = "Invalid credentials" });
        }
    }
}