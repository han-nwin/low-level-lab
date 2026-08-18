" ============================================================
" Vim 8.1 — Windows
" Coding + Markdown Notes
" ============================================================

set nocompatible

" ============================================================
" Core
" ============================================================

filetype plugin indent on
syntax enable

set backspace=indent,eol,start
set hidden
set confirm
set mouse=a

" ============================================================
" Appearance
" ============================================================

set background=dark
colorscheme desert

set number
set relativenumber
set cursorline
set ruler
set showcmd
set showmode
set laststatus=2

set scrolloff=5
set sidescrolloff=5

" ============================================================
" Indentation
" ============================================================

set expandtab
set tabstop=4
set softtabstop=4
set shiftwidth=4
set autoindent
set smartindent

" Code doesn't wrap
set nowrap

" ============================================================
" Search
" ============================================================

set incsearch
set hlsearch
set ignorecase
set smartcase

" Esc clears search highlight
nnoremap <Esc> :nohlsearch<CR>

" ============================================================
" Completion
" ============================================================

set wildmenu
set wildmode=longest:full,full
set path+=**

" ============================================================
" History
" ============================================================

set undolevels=1000
set viminfo='100,<50,s10,h

" ============================================================
" Leader
" ============================================================

let mapleader=" "

" ============================================================
" File Explorer — netrw
"
" Space e = toggle left explorer
" ============================================================

nnoremap <leader>e :Lexplore<CR>

let g:netrw_banner = 0
let g:netrw_liststyle = 3
let g:netrw_winsize = 25

" ============================================================
" Find Files
"
" Space Space
" Type filename / partial path then Tab
" ============================================================

nnoremap <leader><leader> :find **/*

" ============================================================
" Grep
"
" Space /
" ============================================================

set grepprg=findstr\ /S\ /N\ /I

nnoremap <leader>/ :grep 

" Quickfix navigation
nnoremap ]q :cnext<CR>
nnoremap [q :cprevious<CR>

" Open grep results
nnoremap <leader>c :copen<CR>

" Close grep results
nnoremap <leader>C :cclose<CR>

" ============================================================
" Buffers
"
" Space b
" ============================================================

nnoremap <leader>b :buffers<CR>:buffer<Space>

" Next / previous buffer
nnoremap ]b :bnext<CR>
nnoremap [b :bprevious<CR>

" ============================================================
" Splits
" ============================================================

nnoremap <leader>v :vsplit<CR>
nnoremap <leader>s :split<CR>

" Navigate splits
nnoremap <C-h> <C-w>h
nnoremap <C-j> <C-w>j
nnoremap <C-k> <C-w>k
nnoremap <C-l> <C-w>l

" ============================================================
" Visual indentation
" ============================================================

" Keep selection after indenting
vnoremap < <gv
vnoremap > >gv

" ============================================================
" Whitespace
" ============================================================

set listchars=tab:>-,trail:.,extends:>,precedes:<

" Space l = toggle whitespace
nnoremap <leader>l :set list!<CR>

" ============================================================
" Coding
" ============================================================

" C / C++ / Rust use 4 spaces
autocmd FileType c,cpp,rust setlocal tabstop=4 softtabstop=4 shiftwidth=4 expandtab

" ============================================================
" Markdown / Notes
" ============================================================

augroup markdown_notes
    autocmd!

    " Natural note wrapping
    autocmd FileType markdown setlocal wrap
    autocmd FileType markdown setlocal linebreak

    " Markdown = 2 spaces
    autocmd FileType markdown setlocal tabstop=2
    autocmd FileType markdown setlocal softtabstop=2
    autocmd FileType markdown setlocal shiftwidth=2
    autocmd FileType markdown setlocal expandtab

    " Don't hide Markdown characters
    autocmd FileType markdown setlocal conceallevel=0

    " Spellcheck
    autocmd FileType markdown setlocal spell spelllang=en_us

    " --------------------------------------------------------
    " Simple Markdown highlighting
    "
    " Replaces Vim 8.1 Markdown syntax because its old
    " emphasis handling can behave strangely with '*'.
    " --------------------------------------------------------

    autocmd FileType markdown syntax clear

    " # Headings
    autocmd FileType markdown syntax match MyMdHeading /^#\+ .*/

    " - bullets
    autocmd FileType markdown syntax match MyMdBullet /^\s*[-+] /

    " **bold**
    autocmd FileType markdown syntax region MyMdBold start=/\*\*/ end=/\*\*/ keepend

    " `inline code`
    autocmd FileType markdown syntax region MyMdCode start=/`/ end=/`/ keepend

    " ``` code fences
    autocmd FileType markdown syntax region MyMdCodeBlock start=/^```/ end=/^```/ keepend

    " Checkboxes
    autocmd FileType markdown syntax match MyMdTodo /^\s*[-+]\s\+\[ \]/
    autocmd FileType markdown syntax match MyMdDone /^\s*[-+]\s\+\[[xX]\]/

    " Highlighting
    autocmd FileType markdown highlight MyMdHeading cterm=bold
    autocmd FileType markdown highlight MyMdBullet cterm=bold
    autocmd FileType markdown highlight MyMdBold cterm=bold
    autocmd FileType markdown highlight MyMdCode cterm=reverse
    autocmd FileType markdown highlight MyMdCodeBlock cterm=reverse
    autocmd FileType markdown highlight MyMdTodo cterm=bold
    autocmd FileType markdown highlight MyMdDone cterm=bold

augroup END

" ============================================================
" Markdown shortcuts
" ============================================================

" Space x = insert checkbox
autocmd FileType markdown nnoremap <buffer> <leader>x I- [ ] <Esc>

" ============================================================
" Vimrc
" ============================================================

" Space ev = edit config
nnoremap <leader>ev :edit $MYVIMRC<CR>

" Space r = reload config
nnoremap <leader>r :source $MYVIMRC<CR>

" ============================================================
" Restore cursor position
" ============================================================

autocmd BufReadPost *
    \ if line("'\"") > 0 && line("'\"") <= line("$") |
    \   execute "normal! g`\"" |
    \ endif
