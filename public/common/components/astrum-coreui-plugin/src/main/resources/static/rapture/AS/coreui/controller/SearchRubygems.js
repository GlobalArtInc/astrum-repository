
/*global Ext, AS*/

/**
 * Rubygems repository search contribution.
 *
 * @since 3.1
 */
Ext.define('AS.coreui.controller.SearchRubygems', {
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
        id: 'assets.attributes.rubygems.platform',
        group: AS.I18n.get('SearchRubygems_Group'),
        config: {
          format: 'rubygems',
          fieldLabel: AS.I18n.get('SearchRubygems_Platform_FieldLabel'),
          width: 250
        }
      },
      {
        id: 'assets.attributes.rubygems.summary',
        group: AS.I18n.get('SearchRubygems_Group'),
        config: {
          format: 'rubygems',
          fieldLabel: AS.I18n.get('SearchRubygems_Summary_FieldLabel'),
          width: 250
        }
      },
      {
        id: 'assets.attributes.rubygems.description',
        group: AS.I18n.get('SearchRubygems_Group'),
        config: {
          format: 'rubygems',
          fieldLabel: AS.I18n.get('SearchRubygems_Description_FieldLabel'),
          width: 250
        }
      }
    ], me);

    search.registerFilter({
      id: 'rubygems',
      name: 'Rubygems',
      text: AS.I18n.get('SearchRubygems_Text'),
      description: AS.I18n.get('SearchRubygems_Description'),
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
