
/*global Ext, NX*/

/**
 * PyPI repository search contribution.
 *
 * @since 3.1
 */
Ext.define('NX.coreui.controller.SearchPyPi', {
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
        id: 'assets.attributes.pypi.classifiers',
        group: NX.I18n.get('SearchPyPi_Group'),
        config: {
          format: 'pypi',
          fieldLabel: NX.I18n.get('SearchPyPi_Classifiers_FieldLabel'),
          width: 250
        }
      },
      {
        id: 'assets.attributes.pypi.description',
        group: NX.I18n.get('SearchPyPi_Group'),
        config: {
          format: 'pypi',
          fieldLabel: NX.I18n.get('SearchPyPi_Description_FieldLabel'),
          width: 250
        }
      },
      {
        id: 'assets.attributes.pypi.keywords',
        group: NX.I18n.get('SearchPyPi_Group'),
        config: {
          format: 'pypi',
          fieldLabel: NX.I18n.get('SearchPyPi_Keywords_FieldLabel'),
          width: 250
        }
      },
      {
        id: 'assets.attributes.pypi.summary',
        group: NX.I18n.get('SearchPyPi_Group'),
        config: {
          format: 'pypi',
          fieldLabel: NX.I18n.get('SearchPyPi_Summary_FieldLabel'),
          width: 250
        }
      }
    ], me);

    search.registerFilter({
      id: 'pypi',
      name: 'pypi',
      text: NX.I18n.get('SearchPyPi_Text'),
      description: NX.I18n.get('SearchPyPi_Description'),
      readOnly: true,
      criterias: [
        { id: 'format', value: 'pypi', hidden: true },
        { id: 'assets.attributes.pypi.classifiers' },
        { id: 'assets.attributes.pypi.description' },
        { id: 'assets.attributes.pypi.keywords' },
        { id: 'assets.attributes.pypi.summary' }
      ]
    }, me);
  }

});
