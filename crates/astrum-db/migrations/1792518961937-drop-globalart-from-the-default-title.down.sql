UPDATE setting
SET value = jsonb_set(value, '{title}', '"GlobalArt Astrum Repository"'), updated_at = now()
WHERE key = 'ui.settings' AND value ->> 'title' = 'Astrum Repository';
