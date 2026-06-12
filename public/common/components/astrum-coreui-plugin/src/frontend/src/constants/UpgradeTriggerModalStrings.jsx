
import React from 'react';

export default {
  TRIGGER_MODAL: {
    TITLE: 'Proceed with upgrade?',
    DESCRIPTION: <>After beginning the database schema migration process, you will not be able to start Astrum Repository nodes 
    until the process is complete. <strong>We highly recommend backing up your database before proceeding.</strong></>,
    CHECKBOX_LABEL: 'Checking this box indicates that you have backed up your database or are aware of the risks associated ' +
    'with proceeding without doing so.',
    CANCEL: 'Cancel',
    CONTINUE: 'Continue'
  }
};
