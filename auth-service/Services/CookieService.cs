public class CookieService
{
    public void SetCookie(HttpResponse response, string key, string value, SameSiteMode? mode, bool? httpOnly, bool? secure, TimeSpan? maxAge, string path = "/")
    {
        response.Cookies.Append(key, value, new CookieOptions
        {
            SameSite = mode ?? SameSiteMode.Lax,
            HttpOnly = httpOnly ?? false,
            Secure = secure ?? false,
            MaxAge = maxAge ?? TimeSpan.FromHours(1),
            Path = path
        });
    }
}