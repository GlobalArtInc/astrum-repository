
/*global Ext, NX*/

/**
 * Search Criteria store.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.store.SearchCriteria', {
  extend: 'Ext.data.Store',
  model: 'NX.coreui.model.SearchCriteria',
  requires: [
    'NX.I18n'
  ],

  autoLoad: true,

  proxy: {
    type: 'memory',
    reader: {
      type: 'json'
    }
  },

  sorters: { property: 'id', direction: 'ASC' },

  listeners: {
    'beforeSelect': function(sm, record, i, opts) {
      // only allow selection of supported records
      return record.get('supported');
    }
  },

  data: [
    {
      id: 'format',
      config: {
        fieldLabel: NX.I18n.get('SearchCriteria_Format_FieldLabel')
      }
    },
    {
      id: 'keyword',
      config: {
        fieldLabel: NX.I18n.get('SearchCriteria_Keyword_FieldLabel'),
        width: 250
      }
    },
    {
      id: 'repository_name',
      config: {
        fieldLabel: NX.I18n.get('SearchCriteria_RepositoryName_FieldLabel'),
        width: 250
      }
    },
    {
      id: 'version',
      config: {
        fieldLabel: NX.I18n.get('SearchCriteria_Version_FieldLabel')
      }
    },
    {
      id: 'group.raw',
      config: {
        fieldLabel: NX.I18n.get('SearchCriteria_Group_FieldLabel'),
        width: 250
      }
    },
    {
      id: 'name.raw',
      config: {
        fieldLabel: NX.I18n.get('SearchCriteria_Name_FieldLabel'),
        width: 200
      }
    },
    {
      id: 'tags',
      config: {
        fieldLabel: NX.I18n.get('SearchCriteria_Tag_FieldLabel'),
        width: 200
      }
    },
    {
      id: 'assets.attributes.checksum.sha1',
      group: NX.I18n.get('SearchCriteria_Checksum_Group'),
      config: {
        fieldLabel: NX.I18n.get('SearchCriteria_SHA1_FieldLabel'),
        width: 250
      }
    },
    {
      id: 'assets.attributes.checksum.sha256',
      group: NX.I18n.get('SearchCriteria_Checksum_Group'),
      config: {
        fieldLabel: NX.I18n.get('SearchCriteria_SHA256_FieldLabel'),
        width: 250
      }
    },
    {
      id: 'assets.attributes.checksum.sha512',
      group: NX.I18n.get('SearchCriteria_Checksum_Group'),
      config: {
        fieldLabel: NX.I18n.get('SearchCriteria_SHA2_FieldLabel'),
        width: 250
      }
    },
    {
      id: 'assets.attributes.checksum.md5',
      group: NX.I18n.get('SearchCriteria_Checksum_Group'),
      config: {
        fieldLabel: NX.I18n.get('SearchCriteria_MD5_FieldLabel'),
        width: 250
      }
    }
  ]
});
