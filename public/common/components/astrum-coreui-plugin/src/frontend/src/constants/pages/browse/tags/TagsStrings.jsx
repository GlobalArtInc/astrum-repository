
import React from 'react';
import {NxTextLink} from '@globalart/react-shared-components';
import { faTags } from '@fortawesome/free-solid-svg-icons';

export default {
  TAGS: {
    MENU: {
      text: 'Tags',
      description: 'View created tags',
      icon: faTags
    },
    EMPTY_MESSAGE: 'No tags found.',
    HELP_MESSAGE: {
      TITLE: 'What is a tag?',
      TEXT: 
      <>
        Tagging is a Astrum Repository Pro feature that allows you to mark a set of components with a tag so that they can be logically associated with one another.
        You can use tags however you like, but the most common use cases are for CI build IDs for a project (e.g., project-abc-build-142) or for higher-level
        release train when coordinating multiple projects as a single unit (e.g., release-train-13). It is also extensively used by our 
        {" "}<NxTextLink href="http://links.globalart.dev/products/astrum/docs/staging" external>staging</NxTextLink> feature.
        Check out our
        {" "}<NxTextLink href="http://links.globalart.dev/products/astrum/docs/component-tag" external>help documentation</NxTextLink> for
        more information.
      </>,
    },
    LIST: {
      CAPTION: 'Available Tags',
      COLUMNS: {
        NAME: 'Tag Name',
        FIRST_CREATED: 'First created time',
        LAST_UPDATED: 'Last Updated Time',
      },
     FILTER_PLACEHOLDER: 'Filter by Name',
    },
    DETAILS: {
      HEADER: {
        text: 'Tags',
        description: 'View tag details',
      },
      TILE_HEADER: 'Tag Details',
      BACK_TO_TAGS_TABLE: 'Back to Tags Table View',
      ATTRIBUTES: 'Attributes',
      FIND_TAGGED: 'Find Tagged Components',
      FIRST_CREATED: 'First Created Time',
      LAST_UPDATED: 'Last Updated Time',
      TAG_NOT_FOUND: 'Tag was not found'
    }
  }
};
