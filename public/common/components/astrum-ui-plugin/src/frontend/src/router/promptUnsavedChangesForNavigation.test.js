

import { promptUnsavedChangesForNavigation } from './createRouter';
import * as dialog from './unsavedChangesDialog';

describe('promptUnsavedChangesForNavigation', () => {
  afterEach(() => {
    delete window.showPreviewUnsavedDialog;
    jest.restoreAllMocks();
  });

  it('uses the Radix preview dialog and not the classic modal on preview routes when registered', async () => {
    const classicSpy = jest
      .spyOn(dialog, 'showUnsavedChangesModal')
      .mockResolvedValue(true);
    window.showPreviewUnsavedDialog = jest.fn().mockResolvedValue(true);

    const result = await promptUnsavedChangesForNavigation(true);

    expect(window.showPreviewUnsavedDialog).toHaveBeenCalledTimes(1);
    expect(classicSpy).not.toHaveBeenCalled();
    expect(result).toBe(true);
  });

  it('falls back to the classic modal on preview routes when the Radix dialog is unavailable', async () => {
    const classicSpy = jest
      .spyOn(dialog, 'showUnsavedChangesModal')
      .mockResolvedValue(false);
    delete window.showPreviewUnsavedDialog;

    const result = await promptUnsavedChangesForNavigation(true);

    expect(classicSpy).toHaveBeenCalledTimes(1);
    expect(result).toBe(false);
  });

  it('uses the classic modal directly on non-preview routes', async () => {
    const classicSpy = jest
      .spyOn(dialog, 'showUnsavedChangesModal')
      .mockResolvedValue(true);
    window.showPreviewUnsavedDialog = jest.fn().mockResolvedValue(true);

    const result = await promptUnsavedChangesForNavigation(false);

    expect(classicSpy).toHaveBeenCalledTimes(1);
    expect(window.showPreviewUnsavedDialog).not.toHaveBeenCalled();
    expect(result).toBe(true);
  });
});
