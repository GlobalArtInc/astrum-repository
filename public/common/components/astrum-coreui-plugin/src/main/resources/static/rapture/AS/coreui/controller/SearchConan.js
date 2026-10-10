
/**
 * @since 3.28
 */
Ext.define('AS.coreui.controller.SearchConan', {
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
        id: 'attributes.conan.baseVersion',
        group: AS.I18n.get('SearchConan_Group'),
        config: {
          format: 'conan',
          fieldLabel: AS.I18n.get('SearchConan_BaseVersion_FieldLabel'),
          width: 250
        }
      },
      {
        id: 'attributes.conan.channel',
        group: AS.I18n.get('SearchConan_Group'),
        config: {
          format: 'conan',
          fieldLabel: AS.I18n.get('SearchConan_Channel_FieldLabel'),
          width: 250
        }
      },
      {
        id: 'attributes.conan.revision',
        group: AS.I18n.get('SearchConan_Group'),
        config: {
          format: 'conan',
          fieldLabel: AS.I18n.get('SearchConan_RecipeRevision_FieldLabel'),
          width: 250
        }
      },
      {
        id: 'assets.attributes.conan.packageId',
        group: AS.I18n.get('SearchConan_Group'),
        config: {
          format: 'conan',
          fieldLabel: AS.I18n.get('SearchConan_PackageId_FieldLabel'),
          width: 250
        }
      },
      {
        id: 'assets.attributes.conan.packageRevision',
        group: AS.I18n.get('SearchConan_Group'),
        config: {
          format: 'conan',
          fieldLabel: AS.I18n.get('SearchConan_PackageRevision_FieldLabel'),
          width: 250
        }
      },
      {
        id: 'attributes.conan.baseVersion.strict',
        group: AS.I18n.get('SearchConan_Group'),
        config: {
          format: 'conan',
          fieldLabel: AS.I18n.get('SearchConan_BaseVersionStrict_FieldLabel'),
          width: 250
        }
      },
      {
        id: 'attributes.conan.revision.latest',
        group: AS.I18n.get('SearchConan_Group'),
        config: {
          format: 'conan',
          fieldLabel: AS.I18n.get('SearchConan_RecipeRevisionLatest_FieldLabel'),
          width: 250
        }
      },
      {
        id: 'attributes.conan.settings.arch',
        group: AS.I18n.get('SearchConan_Group'),
        config: {
          format: 'conan',
          fieldLabel: AS.I18n.get('SearchConan_Arch_FieldLabel'),
          width: 250
        }
      },
      {
        id: 'attributes.conan.settings.os',
        group: AS.I18n.get('SearchConan_Group'),
        config: {
          format: 'conan',
          fieldLabel: AS.I18n.get('SearchConan_Os_FieldLabel'),
          width: 250
        }
      },
      {
        id: 'attributes.conan.settings.compiler',
        group: AS.I18n.get('SearchConan_Group'),
        config: {
          format: 'conan',
          fieldLabel: AS.I18n.get('SearchConan_Compiler_FieldLabel'),
          width: 250
        }
      },
      {
        id: 'attributes.conan.settings.compiler.version',
        group: AS.I18n.get('SearchConan_Group'),
        config: {
          format: 'conan',
          fieldLabel: AS.I18n.get('SearchConan_CompilerVersion_FieldLabel'),
          width: 250
        }
      },
      {
        id: 'attributes.conan.settings.compiler.runtime',
        group: AS.I18n.get('SearchConan_Group'),
        config: {
          format: 'conan',
          fieldLabel: AS.I18n.get('SearchConan_CompilerRuntime_FieldLabel'),
          width: 250
        }
      }
    ], me);

    search.registerFilter({
      id: 'conan',
      name: 'Conan',
      text: AS.I18n.get('SearchConan_Text'),
      description: AS.I18n.get('SearchConan_Description'),
      readOnly: true,
      criterias: [
        {id: 'format', value: 'conan', hidden: true},
        {id: 'name.raw'},
        {id: 'attributes.conan.baseVersion'},
        {id: 'attributes.conan.baseVersion.strict'},
        {id: 'attributes.conan.channel'},
        {id: 'attributes.conan.revision'},
        {id: 'assets.attributes.conan.packageId'},
        {id: 'assets.attributes.conan.packageRevision'}
      ]
    }, me);
  }
});
