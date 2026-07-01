
/*global Ext, NX*/

/**
 * Rubygems repository search contribution.
 *
 * @since 3.1
 */
Ext.define('NX.coreui.controller.SearchRubygems', {
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
        id: 'assets.attributes.rubygems.platform',
        group: NX.I18n.get('SearchRubygems_Group'),
        config: {
          format: 'rubygems',
          fieldLabel: NX.I18n.get('SearchRubygems_Platform_FieldLabel'),
          width: 250
        }
      },
      {
        id: 'assets.attributes.rubygems.summary',
        group: NX.I18n.get('SearchRubygems_Group'),
        config: {
          format: 'rubygems',
          fieldLabel: NX.I18n.get('SearchRubygems_Summary_FieldLabel'),
          width: 250
        }
      },
      {
        id: 'assets.attributes.rubygems.description',
        group: NX.I18n.get('SearchRubygems_Group'),
        config: {
          format: 'rubygems',
          fieldLabel: NX.I18n.get('SearchRubygems_Description_FieldLabel'),
          width: 250
        }
      }
    ], me);

    search.registerFilter({
      id: 'rubygems',
      name: 'Rubygems',
      text: NX.I18n.get('SearchRubygems_Text'),
      description: NX.I18n.get('SearchRubygems_Description'),
      readOnly: true,
      criterias: [
        { id: 'format', value: 'rubygems', hidden: true },
        { id: 'name.raw' },
        { id: 'version' },
        { id: 'assets.attributes.rubygems.platform' },
        { id: 'assets.attributes.rubygems.summary' },
        { id: 'assets.attributes.rubygems.description' }
      ]
    }, me);
  }

});
