import { useState } from "react";
import { useSearchParams } from "react-router-dom";

export default function useCardSearch({ stringFields, arrayFields = [], startSearch = false }) {
  const [searchParams, setSearchParams] = useSearchParams();

  const initialOptions = {
    ...Object.fromEntries(stringFields.map((f) => [f, searchParams.get(f) ?? ""])),
    ...Object.fromEntries(arrayFields.map((f) => [f, searchParams.getAll(f)])),
  };

  const [pageNumber, setPageNumber] = useState(parseInt(searchParams.get("page") ?? "1"));
  const [searchOptions, setSearchOptions] = useState(initialOptions);
  const [cards, setCards] = useState([]);
  const [loading, setLoading] = useState(false);

  const hasParams = Object.values(initialOptions).some((v) =>
    Array.isArray(v) ? v.length > 0 : v !== ""
  );
  const [shouldSearch, setShouldSearch] = useState(startSearch || hasParams);

  const handleSearchInput = (event, field) => {
    const newState = { ...searchOptions, [field]: event.target.value };
    setSearchOptions(newState);
    setSearchParams({ ...newState, page: "1" });
  };

  const handleArrayInput = (field, event) => {
    const filtered = searchOptions[field].filter((v) => v !== event.target.value);
    const newState = {
      ...searchOptions,
      [field]: event.target.checked ? [...filtered, event.target.value] : filtered,
    };
    setSearchOptions(newState);
    setSearchParams({ ...newState, page: "1" });
  };

  const handlePageChange = (event) => {
    const newPage = parseInt(event.selected) + 1;
    setShouldSearch(true);
    setPageNumber(newPage);
    setSearchParams({ ...searchOptions, page: String(newPage) });
  };

  const triggerSearch = () => {
    setPageNumber(1);
    setShouldSearch(true);
    setSearchParams({ ...searchOptions, page: "1" });
  };

  return {
    cards, setCards,
    loading, setLoading,
    pageNumber,
    shouldSearch, setShouldSearch,
    searchOptions,
    handleSearchInput,
    handleArrayInput,
    handlePageChange,
    triggerSearch,
  };
}
