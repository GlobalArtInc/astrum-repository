

let resolveFn;
let setVisible = () => {};

export function showUnsavedChangesModal() {
  return new Promise((resolve) => {
    resolveFn = resolve;
    setVisible(true);
  });
}

export function handleContinue() {
  resolveFn(true);
  setVisible(false);
}

export function handleCancel() {
  resolveFn(false);
  setVisible(false);
}

export function setDialogSetter(fn) {
  setVisible = fn;
}

// Reset module-level singletons. Intended for test isolation: the dialog
// singleton retains references to a previously mounted component's state
// setter and an unresolved promise's resolver across tests, which can leak
// between test cases under slow CI timing. Test setup calls this between
// cases to guarantee a clean slate.
export function resetDialogState() {
  setVisible(false);
  resolveFn = undefined;
  setVisible = () => {};
}
