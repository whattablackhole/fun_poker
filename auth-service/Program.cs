using Microsoft.EntityFrameworkCore;
using dotenv.net;
using System.Security.Cryptography;


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


builder.Services.AddEndpointsApiExplorer();
builder.Services.AddSwaggerGen();
builder.Services.AddControllers();


builder.Services.AddDbContext<PostgresDbContext>(options =>
    {
        var connectionString = Environment.GetEnvironmentVariable("PostgresConnection");
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

    app.UseCors("AllowOriginDevelopment");
}

app.UseHttpsRedirection();
app.UseRouting();

app.MapControllers();


app.Run();
