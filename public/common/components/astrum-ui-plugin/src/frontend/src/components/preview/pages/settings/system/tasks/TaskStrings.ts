

export const TASK_TYPE_SELECTOR = {
  FILTER_PLACEHOLDER: 'Filter task types...',
  LOADING_MESSAGE: 'Loading task types...',
  COUNT: (count: number) => `${count} task type${count === 1 ? '' : 's'}`,
  EMPTY_FILTER: 'No task types match your filter',
  COLUMN_NAME: 'Name',
  COLUMN_CATEGORY: 'Category',
  COLUMN_DESCRIPTION: 'Description',
};

export const DYNAMIC_FORM_FIELDS = {
  ALL_REPOSITORIES_LABEL: '(All Repositories)',
  ALL_REPOSITORIES_VALUE: '*',
  // Backend BlobStoreComponent.readWithAll() / readNoneGroupEntriesIncludingEntryForAll()
  // emit a synthetic BlobStoreXO whose name is "(All Blob Stores)" and the descriptor
  // uses idMapping("name") — so the persisted value equals the label.
  // Coupling documented in ASTRUM-53356 (2026-06).
  ALL_BLOB_STORES_LABEL: '(All Blob Stores)',
  ALL_BLOB_STORES_VALUE: '(All Blob Stores)',
};
