import localize from '../../../../../../interface/localize';

export const TASK_TYPE_SELECTOR = localize('react-ui', {
  FILTER_PLACEHOLDER: 'Filter task types...',
  LOADING_MESSAGE: 'Loading task types...',
  COUNT: {
    ONE: (count: number) => `${count} task type`,
    FEW: (count: number) => `${count} task types`,
    MANY: (count: number) => `${count} task types`,
    OTHER: (count: number) => `${count} task types`,
  },
  EMPTY_FILTER: 'No task types match your filter',
  COLUMN_NAME: 'Name',
  COLUMN_CATEGORY: 'Category',
  COLUMN_DESCRIPTION: 'Description',
}, 'PREVIEW.TASKS.TYPE_TABLE');

const DYNAMIC_FORM_FIELD_LABELS = localize('react-ui', {
  ALL_REPOSITORIES_LABEL: '(All Repositories)',
  ALL_BLOB_STORES_LABEL: '(All Blob Stores)',
}, 'PREVIEW.TASKS.DYNAMIC_FORM_FIELDS');

export const DYNAMIC_FORM_FIELDS = {
  ALL_REPOSITORIES_LABEL: DYNAMIC_FORM_FIELD_LABELS.ALL_REPOSITORIES_LABEL as string,
  ALL_REPOSITORIES_VALUE: '*',
  // Backend BlobStoreComponent.readWithAll() / readNoneGroupEntriesIncludingEntryForAll()
  // emit a synthetic BlobStoreXO whose name is "(All Blob Stores)" and the descriptor
  // uses idMapping("name") — so the persisted value equals the English label; the label
  // shown is translated, the value is not.
  // Coupling documented in ASTRUM-53356 (2026-06).
  ALL_BLOB_STORES_LABEL: DYNAMIC_FORM_FIELD_LABELS.ALL_BLOB_STORES_LABEL as string,
  ALL_BLOB_STORES_VALUE: '(All Blob Stores)',
};
