
import { faEnvelope } from '@fortawesome/free-solid-svg-icons';

export default {
  EMAIL_SERVER: {
    MENU: {
      text: 'Email Server',
      description: 'Manage email server configuration',
      icon: faEnvelope
    },
    FORM: {
      SECTIONS: {
        SETUP: 'Email Server Configuration',
      },
      ENABLED: {
        LABEL: 'Enable Email Server',
        SUB_LABEL: 'Enabled',
      },
      HOST: {
        LABEL: 'Host',
      },
      PORT: {
        LABEL: 'Port',
      },
      USERNAME: {
        LABEL: 'Username',
      },
      PASSWORD: {
        LABEL: 'Password',
      },
      FROM_ADDRESS: {
        LABEL: 'From Address',
      },
      SUBJECT_PREFIX: {
        LABEL: 'Subject Prefix',
      },
      SSL_TLS_OPTIONS: {
        LABEL: 'SSL/TLS Options',
        OPTIONS: {
          ENABLE_STARTTLS: 'Enable STARTTLS support for insecure connections',
          REQUIRE_STARTTLS: 'Require STARTTLS support',
          ENABLE_SSL_TLS: 'Enable SSL/TLS encryption upon connection',
          IDENTITY_CHECK: 'Enable server identity check',
        },
      },
    },
    VERIFY: {
      LABEL: 'Verify Email Server',
      SUB_LABEL: 'Where do you want to send the test email?',
      TEST: 'Test',
      SUCCESS: 'Email server verification email sent successfully',
      ERROR: 'Email server verification email failed'
    },
    READ_ONLY: {
      ENABLE: {
        ENABLE_STARTTLS: 'STARTTLS support enabled for insecure connections',
        REQUIRE_STARTTLS: 'STARTTLS support required',
        ENABLE_SSL_TLS: 'SSL/TLS encryption enabled upon connection',
        IDENTITY_CHECK: 'Server identity check enabled',
      },
      NOT_ENABLE: {
        ENABLE_STARTTLS: 'STARTTLS support not enabled for insecure connections',
        REQUIRE_STARTTLS: 'STARTTLS support not required',
        ENABLE_SSL_TLS: 'SSL/TLS encryption not enabled upon connection',
        IDENTITY_CHECK: 'Server identity check not enabled',
      }
    }
  }
};