
export const singleNodeResponse1 = {
  'astrum-status': {
    status: 'value1'
  },
  'astrum-node': {
    'node-id': 'nodeId1'
  },
  'astrum-configuration': {
    enabled: true
  },
  'astrum-properties': {
    property: false
  },
  'astrum-license': {
    fingerprint: 'hash'
  },
  'system-time': {
    time: 'value'
  },
  'system-properties': {
    property: 'value1'
  },
  'system-environment': {
    property: 'value1'
  },
  'system-runtime': {
    property: 'value1'
  },
  'system-network': {
    en0: {
      enabled: true
    }
  },
  'system-filestores': {
    '/dev/null': {
      path: '/dev/null'
    }
  }
};

export const singleNodeResponse2 = {
  'astrum-status': {
    status: 'value2'
  },
  'astrum-node': {
    'node-id': 'nodeId2'
  },
  'astrum-configuration': {
    enabled: true
  },
  'astrum-properties': {
    property: true
  },
  'astrum-license': {
    fingerprint: 'hash'
  },
  'system-time': {
    time: 'value'
  },
  'system-properties': {
    property: 'value2'
  },
  'system-environment': {
    property: 'value2'
  },
  'system-runtime': {
    property: 'value2'
  },
  'system-network': {
    en0: {
      enabled: true
    }
  },
  'system-filestores': {
    '/dev/null': {
      path: '/dev/null'
    }
  }
};

export const multiNodeResponse = {
  'astrum-status': {
    nodeId1: {
      status: 'value1'
    },
    nodeId2: {
      status: 'value2'
    }
  },
  'astrum-node': {
    nodeId1: {
      'node-id': 'nodeId1'
    },
    nodeId2: {
      'node-id': 'nodeId2'
    }
  },
  'astrum-configuration': {
    nodeId1: {
      enabled: true
    },
    nodeId2: {
      enabled: true
    }
  },
  'astrum-properties': {
    nodeId1: {
      property: false
    },
    nodeId2: {
      property: true
    }
  },
  'astrum-license': {
    nodeId1: {
      fingerprint: 'hash'
    },
    nodeId2: {
      fingerprint: 'hash'
    }
  },
  'system-time': {
    nodeId1: {
      time: 'value'
    },
    nodeId2: {
      time: 'value'
    }
  },
  'system-properties': {
    nodeId1: {
      property: 'value1'
    },
    nodeId2: {
      property: 'value2'
    }
  },
  'system-environment': {
    nodeId1: {
      property: 'value1'
    },
    nodeId2: {
      property: 'value2'
    }
  },
  'system-runtime': {
    nodeId1: {
      property: 'value1'
    },
    nodeId2: {
      property: 'value2'
    }
  },
  'system-network': {
    nodeId1: {
      en0: {
        enabled: true
      }
    },
    nodeId2: {
      en0: {
        enabled: true
      }
    }
  },
  'system-filestores': {
    nodeId1: {
      '/dev/null': {
        path: '/dev/null'
      }
    },
    nodeId2: {
      '/dev/null': {
        path: '/dev/null'
      }
    }
  }
};
