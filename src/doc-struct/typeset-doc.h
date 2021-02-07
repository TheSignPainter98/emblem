#pragma once

#include "measurement/measurement.h"
#include "doc-ast.h"

typedef struct
{
	const char* content;
	Pos* pos;
	Dimen* dim;
} TTxtData;

typedef struct
{
	Pos* pos;
	Dimen* dim;
	bool isInline;
	const char* srcFname;
	void* data;
} TImgData;

typedef enum
{
	TTXT,
	TIMG
} TLineContentType;

typedef union
{
	TTxtData txt;
	TImgData img;
} TLineContentData;

typedef struct TLineContent_s
{
	struct TLineContent_s* nxt;
	struct TLineContent_s* prv;
	TLineContentType type;
	TLineContentData data;
} TLineContent;

typedef struct TLine_s
{
	struct TLine_s* nxt;
	struct TLine_s* prv;
	Pos* pos;
	Dimen* size;
	TLineContent* fstCont;
	TLineContent* lstCont;
} TLine;

typedef struct TFloater_s
{
	struct TFloater_s* nxt;
	struct TFloater_s* prv;
	Pos* pos;
	Dimen* dim;
	TLine* fstln;
	TLine* lstln;
} TFloater;

typedef struct TPage_s
{
	TPage_s* nxt;
	TPage_s* prv;
	int num;
	Dimen* dim;
	TFloater* fstflt;
	TFloater* lstflt;
	TLine* fstln;
	TLine* lstln;
} TPage;

typedef struct
{
	char* dname;
	char* author;
	char* ver;
	char* date;
	char* modDate;
	char* producer;
	char* subject;
	char* title;
	int kwdsCnt;
	char** kwds;
} TDocMetaData;

typedef struct
{
	TPage* fst;
	TPage* lst;
	int totPgs;
	TDocMetaData meta;
} TDoc;

int ttxt_data_create(TTxtData* txtd, const char* content, Pos* pos, Dimen* dim);
int ttxt_data_destroy(TTxtData* txtd);
int timg_data_create(TImgData* imgd, Pos* pos, Dimen* dim, bool isInline, const char* srcFname, void* data);
int timg_data_destroy(TImgData imgd);
int tline_content_create(TLineContent* cont, TLineContent* nxt, TLineContent* prv, TLineContentType type, TLineContentData data);
int tline_content_destroy(TLineContent* cont);
int tline_create(TLine* ln, TLine* nxt, TLine* prv, Pos* pos, Dimen* dim, TLineContent* fstCont, TLineContent* lstCont);
int tline_destroy(TLine* ln);
int tfloater_create(TFloater* flt, TFloater* nxt, TFloater* prv, Pos* pos, Dimen* dim, TLine* fstln, TLine* lstln);
int tfloater_destroy(TFloater* flt);
int tpage_create(TPage* pg, TPage* pxt, TPage* prv, int num, Dimen* dim, TFloater* fstflt, TFloater* lstflt, TLine* fstln, TLine* lstln);
int tpage_destroy(TPage* pg);
int tdoc_meta_data_create(TDocMetaData* meta, char* dname, char* author, char* ver, char* data, char* modDate, char* producer, char* subject, char* title, int kwdsCnt, char** kwds);
int tdoc_meta_data_destroy(TDocMetaData* meta);
int tdoc_create(TDoc* doc, TPage* fst, TPage* lst, int totPgs, TDocMetaData* meta);
int tdoc_destroy(TDoc* doc);
