-- Add license table
CREATE TABLE IF NOT EXISTS public.license (
    id integer NOT NULL,
    user_id integer NULL,
    name character varying(255) UNIQUE NOT NULL,
    description text NOT NULL,
    url character varying(255) NULL,
    contract text NULL,
    PRIMARY KEY (id),
    FOREIGN KEY (user_id) REFERENCES public.user (id) ON DELETE CASCADE
);

-- Add default_license column to user
ALTER TABLE public.user
    ADD COLUMN IF NOT EXISTS default_license_id integer NULL,
    ADD FOREIGN KEY (default_license_id) REFERENCES public.license (id) ON DELETE SET NULL;

-- Add license column to puzzle
ALTER TABLE public.puzzle
    ADD COLUMN IF NOT EXISTS license_id integer NULL,
    ADD FOREIGN KEY (license_id) REFERENCES public.license (id) ON DELETE SET NULL;

-- Add license objects
INSERT INTO public.license (id, name, description, url)
VALUES 
    ('1', 'CC BY 4.0', E'| 項目 | 制約                          |\n|------|-------------------------------|\n| 署名 | 署名表示が必須 (BY)           |\n| 営利 | 営利目的可能                  |\n| 翻案 | 改変・オマージュ可能          |\n\n**詳しくは「ヘルプ」→「ライセンス」をご覧ください**', 'https://creativecommons.org/licenses/by/4.0/deed.ja'),
    ('2', 'CC BY-SA 4.0', E'| 項目 | 制約                          |\n|------|-------------------------------|\n| 署名 | 署名表示が必須 (BY)           |\n| 継承 | 必ず同じライセンスを使用 (SA) |\n| 営利 | 営利目的可能                  |\n| 翻案 | 改変・オマージュ可能          |\n\n**詳しくは「ヘルプ」→「ライセンス」をご覧ください**', 'https://creativecommons.org/licenses/by-sa/4.0/deed.ja'),
    ('3', 'CC BY-NC 4.0', E'| 項目 | 制約                          |\n|------|-------------------------------|\n| 署名 | 署名表示が必須 (BY)           |\n| 営利 | 営利目的禁止 (NC)             |\n| 翻案 | 改変・オマージュ可能          |\n\n**詳しくは「ヘルプ」→「ライセンス」をご覧ください**', 'https://creativecommons.org/licenses/by-nc/4.0/deed.ja'),
    ('4', 'CC BY-ND 4.0', E'| 項目 | 制約                          |\n|------|-------------------------------|\n| 署名 | 署名表示が必須 (BY)           |\n| 営利 | 営利目的可能                  |\n| 翻案 | 改変・オマージュ禁止 (ND)     |\n\n**詳しくは「ヘルプ」→「ライセンス」をご覧ください**', 'https://creativecommons.org/licenses/by-nd/4.0/deed.ja'),
    ('5', 'CC BY-NC-ND 4.0', E'| 項目 | 制約                          |\n|------|-------------------------------|\n| 署名 | 署名表示が必須 (BY)           |\n| 営利 | 営利目的禁止 (NC)             |\n| 翻案 | 改変・オマージュ禁止 (ND)     |\n\n**詳しくは「ヘルプ」→「ライセンス」をご覧ください**', 'https://creativecommons.org/licenses/by-nc-nd/4.0/deed.ja'),
    ('6', 'CC BY-NC-SA 4.0', E'| 項目 | 制約                          |\n|------|-------------------------------|\n| 署名 | 署名表示が必須 (BY)           |\n| 継承 | 必ず同じライセンスを使用 (SA) |\n| 営利 | 営利目的禁止 (NC)             |\n| 翻案 | 改変・オマージュ可能          |\n\n**詳しくは「ヘルプ」→「ライセンス」をご覧ください**', 'https://creativecommons.org/licenses/by-nc-sa/4.0/deed.ja');
