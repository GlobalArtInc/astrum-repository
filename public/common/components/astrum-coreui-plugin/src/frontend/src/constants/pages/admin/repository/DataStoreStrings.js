
import { faDatabase } from '@fortawesome/free-solid-svg-icons';

export default {
  DATASTORE_CONFIGURATION: {
    MENU: {
      text: 'Data Store',
      description: 'Configure the connection used for the database',
      icon: faDatabase
    },
    FIELDS: {
      jdbcUrlLabel: 'JDBC URL',
      usernameLabel: 'Username',
      passwordLabel: 'Password',
      schemaLabel: 'Schema',
      maxConnectionPoolLabel: 'Maximum Connection Pool Size',
      advancedLabel: 'Advanced Settings'
    },

    MESSAGES: {
      LOAD_ERROR: 'An error occurred while attempting to load the data store connection configuration',
      SAVE_ERROR: 'An error occurred while attempting to save the data store connection configuration',
      SAVE_SUCCESS: 'Data store connection configuration updated'
    }
  }
};
