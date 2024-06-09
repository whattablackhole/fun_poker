using Microsoft.EntityFrameworkCore;
using Microsoft.IdentityModel.Protocols.Configuration;
using dotenv.net;

DotEnv.Load();

var builder = WebApplication.CreateBuilder(args);

builder.Services.AddEndpointsApiExplorer();
builder.Services.AddSwaggerGen();
builder.Services.AddControllers();


builder.Services.AddDbContext<PostgresDbContext>(options =>
    {
        var connectionString = Environment.GetEnvironmentVariable("PostgresConnection");

        Console.WriteLine(connectionString);
        options.UseNpgsql(connectionString);
    });

var secretKey = Environment.GetEnvironmentVariable("JWT_SECRET");

if (secretKey == null)
{
    throw new InvalidConfigurationException("Environment variable jwtsecret is not valid");
}

builder.Services.AddScoped<DataSeeder>();

builder.Services.AddScoped(_ =>
{
    return new TokenService(secretKey);
});

var app = builder.Build();

if (app.Environment.IsDevelopment())
{
    var scope = app.Services.CreateScope();
    var dataSeeder = scope.ServiceProvider.GetRequiredService<DataSeeder>();

    if (dataSeeder != null)
    {
        dataSeeder.SeedData();
    }

    app.UseSwagger();
    app.UseSwaggerUI();
}

app.UseHttpsRedirection();
app.UseRouting();

app.MapControllers();


app.Run();
