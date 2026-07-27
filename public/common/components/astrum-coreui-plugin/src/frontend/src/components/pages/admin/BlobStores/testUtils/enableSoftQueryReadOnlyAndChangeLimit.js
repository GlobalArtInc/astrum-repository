
import {fireEvent} from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import {blobStoreFormSelectors as selectors} from './blobStoreFormSelectors';

export function enableSoftQueryReadOnlyAndChangeLimit(softQuotaLimit) {
  userEvent.click(selectors.softQuota.queryEnabled());
  expect(selectors.softQuota.queryType()).not.toBeInTheDocument();
  expect(selectors.softQuota.querySpaceUsedQuotaReadOnly()).toBeInTheDocument();
  fireEvent.change(selectors.softQuota.queryLimit(), {target: {value: softQuotaLimit}});
  expect(selectors.softQuota.queryLimit()).toHaveValue(softQuotaLimit);
}
