using System.ComponentModel.DataAnnotations;

public class UserRegistrationDto
{
    public required string Name { get; set; }
    public required string Password { get; set; }
    [EmailAddress]
    public required string Email { get; set; }

    public string? CountryCode { get; set; }
}
