

export default function givenBundleActiveStates(state = {}) {
  global.NX.app.Application.bundleActive.mockImplementation((key) => {
    return state[key];
  });
}
