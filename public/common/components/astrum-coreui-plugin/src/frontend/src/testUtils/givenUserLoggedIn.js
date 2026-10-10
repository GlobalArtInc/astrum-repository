

export default function givenUserLoggedIn() {
  global.AS.Security.hasUser.mockReturnValue(true);
}
