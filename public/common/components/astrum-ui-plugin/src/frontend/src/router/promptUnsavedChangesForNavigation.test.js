

import { promptUnsavedChangesForNavigation } from './createRouter';
import * as dialog from './unsavedChangesDialog';

describe('promptUnsavedChangesForNavigation', () => {
  afterEach(() => {
    delete window.showPreviewUnsavedDialog;
    jest.restoreAllMocks();
  });

  it('uses the Radix dialog and not the fallback modal when registered', async () => {
    const classicSpy = jest
      .spyOn(dialog, 'showUnsavedChangesModal')
      .mockResolvedValue(true);
    window.showPreviewUnsavedDialog = jest.fn().mockResolvedValue(true);

    const result = await promptUnsavedChangesForNavigation();

    expect(window.showPreviewUnsavedDialog).toHaveBeenCalledTimes(1);
    expect(classicSpy).not.toHaveBeenCalled();
    expect(result).toBe(true);
  });

  it('falls back to the modal when the Radix dialog is unavailable', async () => {
    const classicSpy = jest
      .spyOn(dialog, 'showUnsavedChangesModal')
      .mockResolvedValue(false);
    delete window.showPreviewUnsavedDialog;

    const result = await promptUnsavedChangesForNavigation();

    expect(classicSpy).toHaveBeenCalledTimes(1);
    expect(result).toBe(false);
  });
});
