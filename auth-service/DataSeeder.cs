using BC = BCrypt.Net.BCrypt;
public class DataSeeder
{
    private readonly PostgresDbContext _dbContext;
    private readonly ILogger _logger;
    public DataSeeder(PostgresDbContext dbContext, ILogger<DataSeeder> logger)
    {
        _dbContext = dbContext;
        _logger = logger;
    }


    public void SeedData()
    {
        if (_dbContext.Users.Any())
        {
            return;
        }
        var users = new List<User>{
            new User {UserName = "user1", Email = "user1@gmail.com", CreatedAt = DateTime.UtcNow, Password = BC.HashPassword("my password1"), Uuid = Guid.NewGuid()},
            new User {UserName = "user2", Email = "user2@gmail.com", CreatedAt = DateTime.UtcNow, Password = BC.HashPassword("my password2"), Uuid = Guid.NewGuid()},
            new User {UserName = "user3", Email = "user3@gmail.com", CreatedAt = DateTime.UtcNow, Password = BC.HashPassword("my password3"), Uuid = Guid.NewGuid()}
        };
        try
        {
            _dbContext.AddRange(users);
            _dbContext.SaveChanges();
        }
        catch (Exception ex)
        {
            _logger.LogError(ex, "An error occurred while adding users to the database");
        }

    }
}