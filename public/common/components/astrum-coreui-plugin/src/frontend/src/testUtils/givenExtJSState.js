

import { ExtJS } from '@globalart/astrum-ui-plugin';

export default   function givenExtJSState(
    values = {},
    edition = "COMMUNITY",
    majorMinorVersion = '1.x.x',
    fullVersion = '1.x.x-snapshot'
) {
  const getValueMock = jest.fn().mockImplementation((key) => {
    return values[key];
  });

  const stateMock = {
    getEdition: jest.fn().mockReturnValue(edition),
    getVersionMajorMinor:jest.fn().mockReturnValue(majorMinorVersion),
    getVersion: jest.fn().mockReturnValue(fullVersion),
    getValue: getValueMock,
    getUser: jest.fn()
  };

  // these resolve to the same thing the real world, but have to be mocked individually
  // because of the way they are accessed
  jest.spyOn(ExtJS, 'state').mockReturnValue(stateMock);
  global.NX.State = stateMock;
  
  // Also mock useStatus for components that use that hook
  jest.spyOn(ExtJS, 'useStatus').mockReturnValue({
    edition: edition,
    version: fullVersion,
  });
}
