
import { ExtJS, copyToClipboard } from '@globalart/astrum-ui-plugin';
import { __, allPass, complement, filter, isEmpty, map, pathEq, prop, propEq } from 'ramda';
import UIStrings from '../../../constants/UIStrings';

const isHosted = propEq('type', 'hosted'),
    isOnline = complement(pathEq(['status', 'online'], false)),
    isNotMavenSnapshotRepo = complement(propEq('versionPolicy', 'SNAPSHOT')),
    isKnownUiUploadableFormat = formats => repo => formats.has(repo.format);

/**
 * Given a list of repos and a list of uploadDefinitions, return a list of repos from the input which
 * support upload through the UI.
 * @param repos a list of repository data structures as returned by the coreui_Repository readReferences ExtDirect
 * API
 */
export function filterReposByUiUpload(uploadDefinitions, repos) {
  const uiUploadableFormats = new Set(map(prop('format'), filter(propEq('uiUpload', true), uploadDefinitions))),
      repoSupportsUpload = allPass([
        isHosted,
        isOnline,
        isNotMavenSnapshotRepo,
        isKnownUiUploadableFormat(uiUploadableFormats)
      ]);

  return filter(repoSupportsUpload, repos);
}

export function repoSupportsUiUpload(uploadDefinitions, repo) {
  return !isEmpty(filterReposByUiUpload(uploadDefinitions, [repo]));
}

/**
 * Copies the provided URL to the clipboard and displays a success or error message.
 *
 * @param {Event} event - The event object to stop propagation.
 * @param {string} url - The URL to be copied to the clipboard.
 */
export async function copyUrl(event, url) {
  event.stopPropagation();
  const success = await copyToClipboard(url);
  if (success) {
    ExtJS.showSuccessMessage(UIStrings.UPLOAD.URL_COPIED_MESSAGE);
  } else {
    ExtJS.showErrorMessage(UIStrings.UPLOAD.URL_COPY_ERROR_MESSAGE);
  }
}
