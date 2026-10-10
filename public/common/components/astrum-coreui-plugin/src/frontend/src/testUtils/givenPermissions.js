

export default function givenPermissions(permissionLookup) {
  global.AS.Permissions.check.mockImplementation((key) => {
    return permissionLookup[key] ?? false;
  });

  global.AS.Permissions.permissions = permissionLookup;
}
