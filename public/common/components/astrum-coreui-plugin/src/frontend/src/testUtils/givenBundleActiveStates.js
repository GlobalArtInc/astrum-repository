

export default function givenBundleActiveStates(state = {}) {
  global.AS.app.Application.bundleActive.mockImplementation((key) => {
    return state[key];
  });
}
