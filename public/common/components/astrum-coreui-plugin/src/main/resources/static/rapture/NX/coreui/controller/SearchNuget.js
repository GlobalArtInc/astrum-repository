
/*global Ext, NX*/

/**
 * NuGet repository search contribution.
 *
 * @since 3.0
 */
Ext.define('NX.coreui.controller.SearchNuget', {
  extend: 'NX.app.Controller',
  requires: [
    'NX.I18n'
  ],

  /**
   * @override
   */
  init: function() {
    var me = this,
        search = me.getController('NX.coreui.controller.Search');

    search.registerCriteria([
      {
        id: 'attributes.nuget.id',
        group: NX.I18n.get('SearchNuget_Group'),
        config: {
          format: 'nuget',
          fieldLabel: NX.I18n.get('SearchNuget_ID_FieldLabel'),
          width: 300
        }
      },
      {
        id: 'assets.attributes.nuget.tags',
        group: NX.I18n.get('SearchNuget_Group'),
        config: {
          format: 'nuget',
          fieldLabel: NX.I18n.get('SearchNuget_Tags_FieldLabel'),
          width: 300
        }
      }
    ], me);

    search.registerFilter({
      id: 'nuget',
      name: 'NuGet',
      text: NX.I18n.get('SearchNuget_Text'),
      description: NX.I18n.get('SearchNuget_Description'),
      readOnly: true,
      criterias: [
        { id: 'format', value: 'nuget', hidden: true },
        { id: 'attributes.nuget.id' },
        { id: 'assets.attributes.nuget.tags' }
      ]
    }, me);
  }

});
