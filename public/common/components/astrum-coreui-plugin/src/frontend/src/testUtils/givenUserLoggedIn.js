

export default function givenUserLoggedIn() {
  global.NX.Security.hasUser.mockReturnValue(true);
}
