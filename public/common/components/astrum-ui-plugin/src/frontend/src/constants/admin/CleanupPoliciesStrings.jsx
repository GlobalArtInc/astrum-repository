import React from 'react';

export default {
  CLEANUP_POLICIES: {
    PREVIEW: {
      BUTTON_TOOLTIP: 'Please select a repository and at least one cleanup criterion',
    },

    DRY_RUN: {
      SAMPLE_COUNT: (count) => `Matching components shown: ${count}. This is only a sample; calculate the totals to see everything the policy would delete.`,
    },

    SUMMARY: {
      TITLE: 'What this policy would delete',
      DESCRIPTION: 'Count every component this policy would delete from the selected repository and the space its files take up. This can take a while on a large repository.',
      BUTTON: 'Calculate',
      CALCULATING: 'Calculating…',
      RESULT: (repository) => <>If it ran now, this policy would delete from <b>{repository}</b>:</>,
      RESULT_AT_LEAST: (repository) => <>If it ran now, this policy would delete at least the following from <b>{repository}</b>:</>,
      COMPONENTS: 'Components',
      FILES: 'Files',
      SIZE: 'Total size',
      INCOMPLETE: 'The calculation stopped at its time limit, so the policy may delete more than this.',
      EMPTY: (repository) => <>Nothing to delete: no component in <b>{repository}</b> matches the criteria.</>,
      ERROR: 'The totals could not be calculated.',
    },
  }
};
