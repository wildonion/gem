



// logout using token_time field
// login, register, edit and avatar apis
// task apis
//  - check new tasks, 
//  - do a task, 
//  - fetch all available tasks, get all tasks
//  - check task that is done (users_tasks table (m:n)) )


/*

    let all_books = books::table.select(Book::as_select()).load(conn)?;
    
    // get all pages for all books
    let pages = Page::belonging_to(&all_books)
        .select(Page::as_select())
        .load(conn)?;

    // group the pages per book
    let pages_per_book = pages
        .grouped_by(&all_books)
        .into_iter()
        .zip(all_books)
        .map(|(pages, book)| (book, pages))
        .collect::<Vec<(Book, Vec<Page>)>>();

*/