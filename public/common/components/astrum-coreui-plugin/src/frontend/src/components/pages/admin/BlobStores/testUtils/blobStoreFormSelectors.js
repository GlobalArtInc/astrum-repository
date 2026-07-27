
import {screen, within} from "@testing-library/react";

export const blobStoreFormSelectors = {
  queryTypeSelect: () => screen.queryByLabelText('Type'),
  queryName: () => screen.queryByLabelText('Name'),
  cancelButton: () => screen.getByText('Cancel'),
  queryConvertToGroupButton: () => screen.queryByText('Convert to Group'),

  // Shared Cloud Selectors
  queryBucket: () => screen.queryByLabelText('Bucket'),
  queryPrefix: () => screen.queryByLabelText('Prefix'),

  softQuota: {
    queryEnabled: () => within(screen.getByRole('group', {name: 'Soft Quota'})).getByLabelText('Enabled'),
    queryType: () => screen.queryByLabelText('Constraint Type'),
    querySpaceUsedQuotaReadOnly: () => screen.queryByText('Space Used'),
    queryLimit: () => screen.queryByLabelText('Constraint Limit (in MB)'),
  }
};
