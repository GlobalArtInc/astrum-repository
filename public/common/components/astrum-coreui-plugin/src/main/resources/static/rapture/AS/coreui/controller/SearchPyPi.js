
/*global Ext, AS*/

/**
 * PyPI repository search contribution.
 *
 * @since 3.1
 */
Ext.define('AS.coreui.controller.SearchPyPi', {
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
        id: 'assets.attributes.pypi.classifiers',
        group: AS.I18n.get('SearchPyPi_Group'),
        config: {
          format: 'pypi',
          fieldLabel: AS.I18n.get('SearchPyPi_Classifiers_FieldLabel'),
          width: 250
        }
      },
      {
        id: 'assets.attributes.pypi.description',
        group: AS.I18n.get('SearchPyPi_Group'),
        config: {
          format: 'pypi',
          fieldLabel: AS.I18n.get('SearchPyPi_Description_FieldLabel'),
          width: 250
        }
      },
      {
        id: 'assets.attributes.pypi.keywords',
        group: AS.I18n.get('SearchPyPi_Group'),
        config: {
          format: 'pypi',
          fieldLabel: AS.I18n.get('SearchPyPi_Keywords_FieldLabel'),
          width: 250
        }
      },
      {
        id: 'assets.attributes.pypi.summary',
        group: AS.I18n.get('SearchPyPi_Group'),
        config: {
          format: 'pypi',
          fieldLabel: AS.I18n.get('SearchPyPi_Summary_FieldLabel'),
          width: 250
        }
      }
    ], me);

    search.registerFilter({
      id: 'pypi',
      name: 'pypi',
      text: AS.I18n.get('SearchPyPi_Text'),
      description: AS.I18n.get('SearchPyPi_Description'),
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
