
/*global Ext, AS*/

/**
 * NuGet repository search contribution.
 *
 * @since 3.0
 */
Ext.define('AS.coreui.controller.SearchNuget', {
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
        id: 'attributes.nuget.id',
        group: AS.I18n.get('SearchNuget_Group'),
        config: {
          format: 'nuget',
          fieldLabel: AS.I18n.get('SearchNuget_ID_FieldLabel'),
          width: 300
        }
      },
      {
        id: 'assets.attributes.nuget.tags',
        group: AS.I18n.get('SearchNuget_Group'),
        config: {
          format: 'nuget',
          fieldLabel: AS.I18n.get('SearchNuget_Tags_FieldLabel'),
          width: 300
        }
      }
    ], me);

    search.registerFilter({
      id: 'nuget',
      name: 'NuGet',
      text: AS.I18n.get('SearchNuget_Text'),
      description: AS.I18n.get('SearchNuget_Description'),
      readOnly: true,
      criterias: [
        { id: 'format', value: 'nuget', hidden: true },
        { id: 'attributes.nuget.id' },
        { id: 'assets.attributes.nuget.tags' }
      ]
    }, me);
  }

});
