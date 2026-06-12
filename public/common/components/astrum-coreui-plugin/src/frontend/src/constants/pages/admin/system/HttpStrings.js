
import { faTruck } from '@fortawesome/free-solid-svg-icons';

export default {
  HTTP: {
    MENU: {
      text: 'HTTP',
      description: 'Manage outbound HTTP/HTTPS configuration',
      icon: faTruck
    },
    CONFIGURATION: {
      USER_AGENT: {
        LABEL: 'User-Agent Customization',
        SUB_LABEL: 'Custom fragment to append to “User-Agent” header in HTTP requests'
      },
      TIMEOUT: {
        LABEL: 'Connection/Socket Timeout',
        SUB_LABEL: 'Time (seconds) to wait for activity before stopping and retrying the connection'
      },
      ATTEMPTS: {
        LABEL: 'Connection/Socket Retry Attempts',
        SUB_LABEL: 'Maximum number of retry attempts if the initial connection attempt suffers a timeout'
      },
      PROXY: {
        LABEL: 'Proxy Settings',
        SUB_LABEL: 'Provide an IP address or DNS name (e.g., proxy-host or 192.168.1.101), not a URL',
        HTTP_HOST: 'HTTP Proxy Host',
        HTTP_PORT: 'HTTP Proxy Port',
        HTTP_CHECKBOX: 'Enable HTTP proxy',
        HTTPS_HOST: 'HTTPS Proxy Host',
        HTTPS_PORT: 'HTTPS Proxy Port',
        HTTPS_CHECKBOX: 'Enable HTTPS proxy',
        HTTP_AUTHENTICATION: 'HTTP Authentication',
        HTTPS_AUTHENTICATION: 'HTTPS Authentication',
        HTTP_AUTH_CHECKBOX: 'Enable HTTP Authentication',
        HTTPS_AUTH_CHECKBOX: 'Enable HTTPS Authentication',
        USERNAME: 'Username',
        PASSWORD: 'Password',
        HOST_NAME: 'Windows NTLM Hostname',
        DOMAIN: 'Windows NTLM Domain'
      },
      EXCLUDE: {
        LABEL: 'Hosts to exclude from HTTP/HTTPS Proxy',
        SUB_LABEL: 'Accepts Java “http.nonProxyHosts” wildcard patterns (one per line, no “l” hostname delimiters)',
        ADD: 'Add',
        REMOVE: 'Remove',
        ALREADY_ADDED: 'The pattern has already been added to the list'
      },
      READ_ONLY: {
        LABEL: 'HTTP Configuration',
        WARNING: 'You are viewing a read-only version of this page. Some fields will not appear if they have not changed from their default values or if HTTP/HTTPS authentication is not enabled. Contact your Administrator if you require edit permissions.'
      }
    }
  }
};