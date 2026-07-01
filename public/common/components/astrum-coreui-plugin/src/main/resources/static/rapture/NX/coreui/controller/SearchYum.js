
/*global Ext, NX*/

/**
 * Yum repository search contribution.
 *
 * @since 3.4
 */
Ext.define('NX.coreui.controller.SearchYum', {
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
        id: 'assets.attributes.yum.architecture',
        group: NX.I18n.get('SearchYum_Group'),
        config: {
          format: 'yum',
          fieldLabel: NX.I18n.get('SearchYum_Architecture_FieldLabel'),
          width: 250
        }
      },
      {
        id: 'attributes.yum.name',
        group: NX.I18n.get('SearchYum_Group'),
        config: {
          format: 'yum',
          fieldLabel: NX.I18n.get('SearchYum_Name_FieldLabel'),
          width: 250
        }
      }
    ], me);

    search.registerFilter({
      id: 'yum',
      name: 'Yum',
      text: NX.I18n.get('SearchYum_Text'),
      description: NX.I18n.get('SearchYum_Description'),
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
