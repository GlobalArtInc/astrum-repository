
/*global Ext, NX*/

/**
 * Composer repository search contribution.
 *
 *
 */
Ext.define('NX.coreui.controller.SearchComposer', {
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
        id: 'assets.attributes.composer.vendor',
        group: NX.I18n.get('SearchComposer_Group'),
        config: {
          format: 'composer',
          fieldLabel: NX.I18n.get('SearchComposer_Vendor_FieldLabel'),
          width: 250
        }
      },
      {
        id: 'assets.attributes.composer.package',
        group: NX.I18n.get('SearchComposer_Group'),
        config: {
          format: 'composer',
          fieldLabel: NX.I18n.get('SearchComposer_Package_FieldLabel'),
          width: 250
        }
      },
      {
        id: 'composer.description',
        group: NX.I18n.get('SearchComposer_Group'),
        config: {
          format: 'composer',
          fieldLabel: NX.I18n.get('SearchComposer_Description_FieldLabel'),
          width: 250
        }
      },
      {
        id: 'composer.keywords',
        group: NX.I18n.get('SearchComposer_Group'),
        config: {
          format: 'composer',
          fieldLabel: NX.I18n.get('SearchComposer_Keywords_FieldLabel'),
          width: 250
        }
      }
    ], me);

    search.registerFilter({
      id: 'composer',
      name: 'Composer',
      text: NX.I18n.get('SearchComposer_Text'),
      description: NX.I18n.get('SearchComposer_Description'),
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
