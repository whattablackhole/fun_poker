public class UserDto
{
    public string Id { get; set; }
    public string Name { get; set; }
    public string? CountryCode { get; set; }

     public UserDto(User user)
    {
        Id = user.Id.ToString();
        Name = user.Name;
        CountryCode = user.CountryCode;
    }
}
