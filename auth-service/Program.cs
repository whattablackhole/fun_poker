using Microsoft.EntityFrameworkCore;
using dotenv.net;
using System.Security.Cryptography;
using Microsoft.AspNetCore.HttpOverrides;


DotEnv.Load(options: new DotEnvOptions(envFilePaths: ["../.env"]));

static RSAParameters DecodeRSAPrivateKey(string privateKeyBytes, string password)
{
    using (var rsa = RSA.Create())
    {
        rsa.ImportFromEncryptedPem(privateKeyBytes.ToCharArray(), password);
        return rsa.ExportParameters(true);
    }
}

var builder = WebApplication.CreateBuilder(args);


builder.Services.Configure<ForwardedHeadersOptions>(options =>
{
    options.ForwardedHeaders =
        ForwardedHeaders.XForwardedFor | ForwardedHeaders.XForwardedProto;
});


builder.Services.AddEndpointsApiExplorer();
builder.Services.AddSwaggerGen();
builder.Services.AddControllers();


builder.Services.AddDbContext<PostgresDbContext>(options =>
    {
        var connectionString = Environment.GetEnvironmentVariable("POSTGRES_CONNECTION");
        options.UseNpgsql(connectionString);
    });

builder.Services.AddScoped<DataSeeder>();


RSAParameters privateKeyParams;

string pemKey = File.ReadAllText("private_key.pem");

string password = Environment.GetEnvironmentVariable("ENCRYPTION_PASS")!;

privateKeyParams = DecodeRSAPrivateKey(pemKey, password);


builder.Services.AddScoped(_ =>
{
    return new TokenService(privateKeyParams);
});

builder.Services.AddScoped(_ =>
{
    return new CookieService();
});

builder.Services.AddCors(options =>
{

    options.AddPolicy("AllowOriginDevelopment",
        builder =>
        {
            builder.WithOrigins(["https://localhost:5173", "https://localhost:5174"])
                   .AllowAnyHeader()
                   .AllowAnyMethod()
                   .AllowCredentials();
        });

    options.AddPolicy("Production",
   builder =>
   {
       builder.WithOrigins(["https://localhost:8082"])
              .AllowAnyHeader()
              .AllowAnyMethod()
              .AllowCredentials();
   });
});




var app = builder.Build();

app.UseForwardedHeaders();

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

    app.UseCors("AllowOriginDevelopment");
}
else
{
    app.UseCors("Production");
    // temp solution. have to be done as seperate step at docker level
    using (var scope = app.Services.CreateScope())
    {
        var services = scope.ServiceProvider;

        var context = services.GetRequiredService<PostgresDbContext>();
        if (context.Database.GetPendingMigrations().Any())
        {
            context.Database.Migrate();
        }
    }
}
app.UseHttpsRedirection();
app.UseRouting();

app.MapControllers();


app.Run();
