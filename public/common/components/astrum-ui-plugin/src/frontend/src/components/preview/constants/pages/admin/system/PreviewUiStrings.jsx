
import { faEye } from '@fortawesome/free-solid-svg-icons';
import localize from '../../../../../../interface/localize';

export default localize('react-ui', {
  PREVIEW_UI_SETTINGS: {
    title: 'Astrum One UI Settings',
    MENU: {
      text: 'Astrum One UI',
      description: 'Configure access to the Astrum One UI experience',
      icon: faEye,
    },
    description: 'Control access to the Astrum One UI experience. The Astrum One UI provides a modern, redesigned interface with improved usability and new features.',
    INFO: {
      reloadNote: 'Changes to these settings take effect after the page reloads.',
    },
    ACTIONS: {
      discard: 'Discard Changes',
      save: 'Save',
      unsavedChanges: 'Unsaved changes',
      loading: 'Loading...',
    },
    ACCESS_CONTROL: {
      title: 'Access Control',
    },
    ANONYMOUS: {
      label: 'Anonymous Users',
      description: 'Allow anonymous (non-authenticated) users to access the Astrum One UI.',
      property: 'astrum.preview.ui.anonymous.enabled',
    },
    LOGGEDIN: {
      label: 'Logged-in Users',
      description: 'Allow authenticated users to access the Astrum One UI.',
      property: 'astrum.preview.ui.loggedin.enabled',
    },
    ROLLOUT: {
      title: 'Rollout Control',
      DEFAULT_TO_PREVIEW: {
        label: 'Default to Astrum One UI',
        helpText: 'When enabled, all users land on the Astrum One UI after login. Users can still switch back to the Classic UI at any time using the toggle in the page header.',
        property: 'preview.ui.default.enabled',
      },
      DISABLE_SWITCH_FEEDBACK: {
        label: 'Disable Switch Feedback',
        helpText: 'Hide the feedback prompt and prevent Astrum One UI from sending feedback when users switch to the Classic UI.',
        property: 'preview.ui.switch.feedback.disabled',
      },
    },
    DISPOSAL: {
      title: 'Disable Classic UI',
      DISABLE_LEGACY: {
        label: 'Disable Classic UI',
        helpText: 'Completely remove access to the Classic UI (ExtJS). The Astrum One UI becomes the only available interface, and the header toggle to switch back is removed.',
        warningText: 'CAUTION: When enabled, all users lose access to the Classic UI immediately. If users encounter bugs or missing features in the Astrum One UI, they will have no fallback until an administrator disables this setting.',
        property: 'preview.ui.legacy.disabled',
      },
    },
    UNSAVED_DIALOG: {
      title: 'Unsaved Changes',
      description: 'You have unsaved changes. Are you sure you want to leave? Your changes will be lost.',
      stay: 'Stay',
      leave: 'Leave',
    },
    MESSAGES: {
      lockoutError: 'Cannot disable Classic UI when both Anonymous and Logged-in access to Astrum One UI are disabled. At least one UI access method must remain enabled to prevent lockout.',
      saveFailed: 'Failed to save Astrum One UI settings',
    },
  },
}, 'PREVIEW');
