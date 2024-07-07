using Microsoft.EntityFrameworkCore;
using System.ComponentModel.DataAnnotations;
public class PostgresDbContext : DbContext
{
    public PostgresDbContext(DbContextOptions<PostgresDbContext> options) : base(options) { }

    public DbSet<User> Users { get; set; }

    public DbSet<RefreshToken> RefreshTokens { get; set; }

}

public class User
{
    [Key]
    public int Id { get; set; }

    public string? GoogleId { get; set; }
    public required string Name { get; set; }

    [EmailAddress]
    public required string Email { get; set; }

    public string? CountryCode { get; set; }

    public string? Password { get; set; }
    public DateTime CreatedAt { get; set; }
}

public class RefreshToken
{
    public int Id { get; set; }
    public string Token { get; set; }

    public DateTime Expires { get; set; }

    public bool IsExpired => DateTime.UtcNow >= Expires;
    public DateTime Created { get; set; }

    public DateTime? Revoked { get; set; }

    public bool IsActive => Revoked == null && !IsExpired;

    public int UserId { get; set; }
}