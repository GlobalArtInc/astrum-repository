
/*global Ext, AS*/

/**
 * Yum repository search contribution.
 *
 * @since 3.4
 */
Ext.define('AS.coreui.controller.SearchYum', {
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
        id: 'assets.attributes.yum.architecture',
        group: AS.I18n.get('SearchYum_Group'),
        config: {
          format: 'yum',
          fieldLabel: AS.I18n.get('SearchYum_Architecture_FieldLabel'),
          width: 250
        }
      },
      {
        id: 'attributes.yum.name',
        group: AS.I18n.get('SearchYum_Group'),
        config: {
          format: 'yum',
          fieldLabel: AS.I18n.get('SearchYum_Name_FieldLabel'),
          width: 250
        }
      }
    ], me);

    search.registerFilter({
      id: 'yum',
      name: 'Yum',
      text: AS.I18n.get('SearchYum_Text'),
      description: AS.I18n.get('SearchYum_Description'),
      readOnly: true,
      criterias: [
        { id: 'format', value: 'yum', hidden: true },
        { id: 'attributes.yum.name' },
        { id: 'version' },
        { id: 'assets.attributes.yum.architecture' }
      ]
    }, me);
  }

});
