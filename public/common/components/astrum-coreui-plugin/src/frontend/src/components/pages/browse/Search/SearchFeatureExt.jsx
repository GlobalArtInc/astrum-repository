

import React from "react";
import PropTypes from "prop-types";

import { ExtJsContainer } from "../../../widgets/ExtJsContainer/ExtJsContainer";
import { ExtJS } from "@globalart/astrum-ui-plugin";
export default function SearchFeatureExt({
  title,
  icon,
  criterias,
  filter
}) {
  const keywordFilter = ExtJS.useSearchFilterModel(filter);
  const extParams = { searchFilter: keywordFilter, bookmarkEnding: "" };

  if (criterias) {
    ExtJS.useCriteria(criterias);
  }

  return (
    <ExtJsContainer
      title={title}
      icon={icon}
      className="as-search"
      extView="AS.coreui.view.search.SearchFeature"
      extParams={extParams}
    />
  );
}

SearchFeatureExt.defaultProps = {
  filter: {
    id: "keyword",
    name: "Keyword",
    text: "Keyword",
    description: "Search for components by keyword",
    readOnly: true,
    criterias: [{ id: "keyword" }],
  },
};

SearchFeatureExt.propTypes = {
  criterias: PropTypes.arrayOf(
    PropTypes.shape({
      id: PropTypes.string,
      group: PropTypes.string,
      config: PropTypes.shape({
        format: PropTypes.string,
        fieldLabel: PropTypes.string,
        width: PropTypes.number,
      }),
    })
  ),
  filter: PropTypes.shape({
    id: PropTypes.string,
    name: PropTypes.string,
    text: PropTypes.string,
    description: PropTypes.string,
    readOnly: PropTypes.bool,
    criterias: PropTypes.arrayOf(
      PropTypes.shape({
        id: PropTypes.string,
        value: PropTypes.string,
        hidden: PropTypes.bool,
      })
    ),
  }).isRequired,
};

SearchFeatureExt.propTypes = {
  title: PropTypes.string.isRequired,
  icon: PropTypes.object.isRequired,
  criterias: PropTypes.arrayOf(PropTypes.object),
  filter: PropTypes.object
};
