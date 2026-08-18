" ============================================================
" Leader
" ============================================================

let mapleader=" "

" ------------------------------------------------------------
" File Explorer — netrw
" ------------------------------------------------------------

" Space e = file explorer
nnoremap <leader>e :Explore<CR>

" Space E = explorer at current file
nnoremap <leader>E :Lexplore<CR>

" Netrw appearance
let g:netrw_banner = 0
let g:netrw_liststyle = 3
let g:netrw_browse_split = 0
let g:netrw_winsize = 25


" ------------------------------------------------------------
" Find files
" ------------------------------------------------------------

" Space Space = recursively find file
"
" Type part of filename then Tab for completion:
"   <Space><Space>
"   **/*.cpp<Tab>
"
nnoremap <leader><leader> :find **/*


" ------------------------------------------------------------
" Grep
" ------------------------------------------------------------

" Use Windows built-in findstr
set grepprg=findstr\ /S\ /N\ /I

" Space / = grep project
"
" Example:
"   <Space>/
"   SensorManager
"
nnoremap <leader>/ :grep 


" ------------------------------------------------------------
" Quickfix navigation
" ------------------------------------------------------------

" Grep results go into quickfix
"
" ]q = next result
" [q = previous result
" Space c = open result list

nnoremap ]q :cnext<CR>
nnoremap [q :cprevious<CR>
nnoremap <leader>c :copen<CR>


" ------------------------------------------------------------
" Buffers
" ------------------------------------------------------------

" Space b = list buffers
nnoremap <leader>b :buffers<CR>:buffer<Space>


" ------------------------------------------------------------
" Splits
" ------------------------------------------------------------

nnoremap <leader>v :vsplit<CR>
nnoremap <leader>s :split<CR>

nnoremap <C-h> <C-w>h
nnoremap <C-j> <C-w>j
nnoremap <C-k> <C-w>k
nnoremap <C-l> <C-w>l


" ------------------------------------------------------------
" Config
" ------------------------------------------------------------

" Space ev = edit vimrc
nnoremap <leader>ev :edit $MYVIMRC<CR>

" Space r = reload vimrc
nnoremap <leader>r :source $MYVIMRC<CR>
