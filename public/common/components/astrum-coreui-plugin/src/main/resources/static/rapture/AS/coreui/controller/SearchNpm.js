
/*global Ext, AS*/

/**
 * npm repository search contribution.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.controller.SearchNpm', {
  extend: 'AS.app.Controller',
  requires: [
    'AS.I18n'
  ],

  /**
   * @override
   */
  init: function() {
    var me = this,
        search = me.getController('AS.coreui.controller.Search');

    search.registerCriteria([
      {
        id: 'group',
        group: AS.I18n.get('SearchNpm_Group'),
        config: {
          format: 'npm',
          fieldLabel: AS.I18n.get('SearchNpm_Scope_FieldLabel'),
          width: 250
        }
      },
      {
        id: 'assets.attributes.npm.author',
        group: AS.I18n.get('SearchNpm_Group'),
        config: {
          format: 'npm',
          fieldLabel: AS.I18n.get('SearchNpm_Author_FieldLabel'),
          width: 250
        }
      },
      {
        id: 'assets.attributes.npm.description',
        group: AS.I18n.get('SearchNpm_Group'),
        config: {
          format: 'npm',
          fieldLabel: AS.I18n.get('SearchNpm_Description_FieldLabel'),
          width: 250
        }
      },
      {
        id: 'assets.attributes.npm.keywords',
        group: AS.I18n.get('SearchNpm_Group'),
        config: {
          format: 'npm',
          fieldLabel: AS.I18n.get('SearchNpm_Keywords_FieldLabel'),
          width: 250
        }
      },
      {
        id: 'assets.attributes.npm.license',
        group: AS.I18n.get('SearchNpm_Group'),
        config: {
          format: 'npm',
          fieldLabel: AS.I18n.get('SearchNpm_License_FieldLabel'),
          width: 250
        }
      }
    ], me);

    search.registerFilter({
      id: 'npm',
      name: 'npm',
      text: AS.I18n.get('SearchNpm_Text'),
      description: AS.I18n.get('SearchNpm_Description'),
      readOnly: true,
      criterias: [
        { id: 'format', value: 'npm', hidden: true },
        { id: 'group' },
        { id: 'name.raw' },
        { id: 'version' },
        { id: 'assets.attributes.npm.author' },
        { id: 'assets.attributes.npm.description' },
        { id: 'assets.attributes.npm.keywords' },
        { id: 'assets.attributes.npm.license' }
      ]
    }, me);
  }

});
