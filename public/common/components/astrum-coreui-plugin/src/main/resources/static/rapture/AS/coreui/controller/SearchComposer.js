
/*global Ext, AS*/

/**
 * Composer repository search contribution.
 *
 *
 */
Ext.define('AS.coreui.controller.SearchComposer', {
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
        id: 'assets.attributes.composer.vendor',
        group: AS.I18n.get('SearchComposer_Group'),
        config: {
          format: 'composer',
          fieldLabel: AS.I18n.get('SearchComposer_Vendor_FieldLabel'),
          width: 250
        }
      },
      {
        id: 'assets.attributes.composer.package',
        group: AS.I18n.get('SearchComposer_Group'),
        config: {
          format: 'composer',
          fieldLabel: AS.I18n.get('SearchComposer_Package_FieldLabel'),
          width: 250
        }
      },
      {
        id: 'composer.description',
        group: AS.I18n.get('SearchComposer_Group'),
        config: {
          format: 'composer',
          fieldLabel: AS.I18n.get('SearchComposer_Description_FieldLabel'),
          width: 250
        }
      },
      {
        id: 'composer.keywords',
        group: AS.I18n.get('SearchComposer_Group'),
        config: {
          format: 'composer',
          fieldLabel: AS.I18n.get('SearchComposer_Keywords_FieldLabel'),
          width: 250
        }
      }
    ], me);

    search.registerFilter({
      id: 'composer',
      name: 'Composer',
      text: AS.I18n.get('SearchComposer_Text'),
      description: AS.I18n.get('SearchComposer_Description'),
      readOnly: true,
      criterias: [
        { id: 'format', value: 'composer', hidden: true },
        { id: 'assets.attributes.composer.vendor' },
        { id: 'assets.attributes.composer.package' },
        { id: 'version' },
        { id: 'composer.description' },
        { id: 'composer.keywords' }
      ]
    }, me);
  }

});
