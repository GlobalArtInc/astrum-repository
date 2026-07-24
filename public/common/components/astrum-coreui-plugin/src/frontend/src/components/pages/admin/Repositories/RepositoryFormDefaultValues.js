
export const genericDefaultValues = {
  proxy: {
    type: 'proxy',
    name: '',
    online: true,
    routingRule: '',
    storage: {
      blobStoreName: '',
      strictContentTypeValidation: true
    },
    cleanup: null,
    proxy: {
      remoteUrl: '',
      contentMaxAge: -1,
      metadataMaxAge: 1440,
      preserveEncodedCharacters: false
    },
    negativeCache: {
      enabled: true,
      timeToLive: 1440
    },
    httpClient: {
      blocked: false,
      autoBlock: true,
      connection: null,
      authentication: null
    },
    replication: {
      preemptivePullEnabled: false,
      assetPathRegex: ''
    }
  },
  hosted: {
    type: 'hosted',
    name: '',
    online: true,
    storage: {
      blobStoreName: '',
      strictContentTypeValidation: true,
      writePolicy: 'ALLOW_ONCE'
    },
    component: {
      proprietaryComponents: false
    },
    cleanup: null,
    // TODO: CLM-38706 - Add evaluation field integration with backend API
    evaluation: {
      mode: 'INHERIT'
    }
  },
  group: {
    type: 'group',
    name: '',
    online: true,
    storage: {
      blobStoreName: '',
      strictContentTypeValidation: true
    },
    group: {
      memberNames: []
    }
  }
};
