

export default function givenPermissions(permissionLookup) {
  global.NX.Permissions.check.mockImplementation((key) => {
    return permissionLookup[key] ?? false;
  });

  global.NX.Permissions.permissions = permissionLookup;
}
