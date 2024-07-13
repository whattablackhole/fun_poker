using System.IdentityModel.Tokens.Jwt;
using System.Security.Claims;
using System.Security.Cryptography;
using Microsoft.IdentityModel.Tokens;


public class TokenService
{
    private readonly RSAParameters _signingKey;

    public TokenService(RSAParameters secretKey)
    {
        _signingKey = secretKey;
    }

    public string GenerateToken(User user)
    {
        var tokenHandler = new JwtSecurityTokenHandler
        {
            MapInboundClaims = false
        };
        var tokenDescriptor = new SecurityTokenDescriptor
        {
            Subject = new ClaimsIdentity([
                new Claim(ClaimTypes.Name, user.Name),
                new Claim(ClaimTypes.Email, user.Email),
                new Claim(ClaimTypes.Country, user.CountryCode ?? ""),
                new Claim(JwtRegisteredClaimNames.Sub, user.Id.ToString()),
                new Claim(ClaimTypes.Role, "user")
            ]),
            Issuer = "https://auth.funpoker.com",
            Audience = "https://api.funpoker.com",
            Expires = DateTime.UtcNow.AddHours(3),
            SigningCredentials = new SigningCredentials(new RsaSecurityKey(_signingKey), SecurityAlgorithms.RsaSha256),
        };

        var token = tokenHandler.CreateToken(tokenDescriptor);
        return tokenHandler.WriteToken(token);
    }

    public RefreshToken GenerateRefreshTokenEntity(int userId)
    {
        var refreshToken = GenerateRandomToken();

        return new RefreshToken
        {
            Token = refreshToken,
            Expires = DateTime.UtcNow.AddDays(7),
            Created = DateTime.UtcNow,
            UserId = userId
        };
    }

    public string GenerateRandomToken()
    {
        var randomNumber = new byte[32];
        using (var rng = RandomNumberGenerator.Create())
        {
            rng.GetBytes(randomNumber);
            return Convert.ToBase64String(randomNumber);
        }
    }


    public string GenerateUnauthorizedToken(IEnumerable<Claim> claims)
    {
        var tokenHandler = new JwtSecurityTokenHandler();

        var tokenDescriptor = new SecurityTokenDescriptor
        {
            Subject = new ClaimsIdentity(claims),
            Expires = DateTime.UtcNow.AddDays(1),
            SigningCredentials = new SigningCredentials(new RsaSecurityKey(_signingKey), SecurityAlgorithms.RsaSha256),
        };

        var token = tokenHandler.CreateToken(tokenDescriptor);
        return tokenHandler.WriteToken(token);
    }

    public async Task<TokenValidationResult> ValidateTokenAsync(string token)
    {
        var tokenHandler = new JwtSecurityTokenHandler
        {
            MapInboundClaims = false
        };

        var validationParameters = new TokenValidationParameters
        {
            ValidateIssuerSigningKey = true,
            IssuerSigningKey = new RsaSecurityKey(_signingKey),
            ValidateLifetime = true,
            LifetimeValidator = ValidateLifetime,
            ValidateIssuer = true,
            ValidIssuer = "https://auth.funpoker.com",
            ValidateAudience = true,
            ValidAudience = "https://api.funpoker.com",
        };

        return await tokenHandler.ValidateTokenAsync(token, validationParameters);
    }

    public ClaimsPrincipal ValidateTokenIssuer(string token)
    {
        var tokenHandler = new JwtSecurityTokenHandler
        {
            MapInboundClaims = false
        };

        var validationParameters = new TokenValidationParameters
        {
            ValidateIssuerSigningKey = true,
            IssuerSigningKey = new RsaSecurityKey(_signingKey),
        };

        return tokenHandler.ValidateToken(token, validationParameters, out _);
    }

    public ClaimsPrincipal ValidateTokenExpiration(string token)
    {
        var tokenHandler = new JwtSecurityTokenHandler
        {
            MapInboundClaims = false
        };
        SecurityToken validatedToken;

        var validationParameters = new TokenValidationParameters
        {
            ValidateIssuerSigningKey = true,
            IssuerSigningKey = new RsaSecurityKey(_signingKey),

            ValidateLifetime = true,
            LifetimeValidator = ValidateLifetime
        };

        return tokenHandler.ValidateToken(token, validationParameters, out validatedToken);
    }

    private bool ValidateLifetime(DateTime? notBefore, DateTime? expires, SecurityToken securityToken, TokenValidationParameters validationParameters)
    {
        if (expires != null)
        {
            return DateTime.UtcNow < expires;
        }
        return false;
    }
}