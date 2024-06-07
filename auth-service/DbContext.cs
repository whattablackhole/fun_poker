using Microsoft.EntityFrameworkCore;
using System.ComponentModel.DataAnnotations;
public class PostgresDbContext : DbContext
{
    public PostgresDbContext(DbContextOptions<PostgresDbContext> options) : base(options) { }

    public DbSet<User> Users { get; set; }
}

public class User
{
   [Key]
   public int Id { get; set; }

    public Guid Uuid { get; set; } = Guid.NewGuid();
    public required string UserName { get; set; }
    public required string Email { get; set; }

    public required string Password { get; set; }
    public DateTime CreatedAt { get; set; }
}