import React from "react";
import ReactPaginate from "react-paginate";

export default function SearchPagination({ cards, pageSize, pageNumber, onPageChange }) {
  const pageCount = cards.length >= pageSize ? pageNumber + 1 : pageNumber;

  if (cards.length === 0 || pageCount <= 1) return null;

  return (
    <ReactPaginate
      previousLabel="Previous"
      nextLabel="Next"
      pageClassName="page-item"
      pageLinkClassName="page-link"
      previousClassName="page-item"
      previousLinkClassName="page-link"
      nextClassName="page-item"
      nextLinkClassName="page-link"
      breakLabel="..."
      breakClassName="page-item"
      breakLinkClassName="page-link"
      containerClassName="pagination"
      activeClassName="active"
      pageCount={pageCount}
      marginPagesDisplayed={2}
      pageRangeDisplayed={5}
      onPageChange={onPageChange}
      forcePage={Math.max(0, pageNumber - 1)}
    />
  );
}
