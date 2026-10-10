
import React from 'react';
import {AsTextLink} from '@globalart/react-shared-components';
import { faLayerGroup } from '@fortawesome/free-solid-svg-icons';

export default {
  CONTENT_SELECTORS: {
    MENU: {
      text: 'Content Selectors',
      description: 'Define the content that users can access using Content Selector Expression Language (CSEL)',
      icon: faLayerGroup
    },

    EDIT_TITLE: (name) => `Edit ${name}`,

    EMPTY_MESSAGE: 'No content selectors were found',

    HELP_TITLE: 'What is a content selector?',
    HELP_TEXT: <>
      Content selectors provide a means for you to select specific content from your repositories.
      Repository content is evaluated against expressions written in CSEL (Content Selector Expression Language).
      For more information,{' '}
      <AsTextLink external href="http://links.globalart.dev/products/astrum/docs/content-selector">
        check the documentation
      </AsTextLink>.
    </>,
    CREATE_BUTTON: 'Create Selector',
    FILTER_PLACEHOLDER: 'Filter',

    NAME_LABEL: 'Name',
    TYPE_LABEL: 'Type',
    DESCRIPTION_LABEL: 'Description',
    EXPRESSION_LABEL: 'Search Expression',
    EXPRESSION_DESCRIPTION: 'Use the following query to identify repository content',
    EXPRESSION_EXAMPLES: 'Example Content Selector Expressions',
    RAW_EXPRESSION_EXAMPLE_LABEL: 'Select "raw" format content',
    MULTI_EXPRESSIONS_EXAMPLE_LABEL: 'Select "maven2" content along a path that starts with "/org/"',

    PREVIEW: {
      TITLE: 'Preview Content Selector Results',
      REPOSITORY_LABEL: 'Preview Repository',
      REPOSITORY_DESCRIPTION: 'Select a repository to evaluate the content selector and see the content that would be available',
      BUTTON: 'Preview',
      RESULTS: 'Preview Results',
      NAME_COLUMN: 'Name',
      EMPTY: 'No content in repositories matched the expression'
    },

    MESSAGES: {
      DUPLICATE_ERROR: (name) => `Another content selector named ${name} already exists`,
      NAME_TOO_LONG: 'The name of this content selector is too long, please use a shorter name',
      SAVE_ERROR: 'An error occurred while saving the content selector',
      DELETE_ERROR: (name) => `Content selector ${name} is in use and cannot be deleted`,

      CONFIRM_DELETE: {
        TITLE: 'Delete Content Selector',
        MESSAGE: (name) => `Delete the content selector named ${name}?`,
        YES: 'Delete',
        NO: 'Cancel'
      }
    }
  }
};
