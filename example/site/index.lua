local components = require('components.lua')

-- get all index pages
local pagelinks = {}
for key, value in pairs(template.colocated.files) do
  -- if it's markdown, and not the index page, include it
  if value.extention == "md" and value.stem ~= "index" then 
    local front = value:parseMd().front
    
    -- link to the page
    pagelinks["/" .. value.stem] = front.title 
  end
end

-- load all colocated markdown files
local markdown = {}
for key, value in pairs(template.colocated.files) do
  -- if it's markdown, and not the index page, include it
  if value.extention == "md" and value.stem ~= "index" then
    -- render it to html
    local md = value:parseMd()

    -- get the front matter for the title of the page
    local front = md.front

    -- render out
    local html = components.page(front.title, "", "/style.css", pagelinks, rawHtml(md.html))

    -- and the actual page file
    markdown[value.stem] = page()
      :withHtml(html:renderHtml())
  end
end

-- index page
local html = components.page(
  "YASSG", "", "/style.css", pagelinks, 
  h.main():sub("Hello <$> world!")
):renderHtml()

return page()
  :withHtml(html)
  :withManyPages(markdown)
  :withFile("style.css", template.styles.style)
