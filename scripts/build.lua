-- Get method.
local fw = args.fw
local method = nil

if fw == '11.00' then
  method = args.dm or 'direct'
end

-- Build env.
local env = {
  RUSTFLAGS = string.format(
    '--cfg fw="%s" --cfg method="%s" -Z unstable-options -C panic=immediate-abort',
    fw:gsub('%.', ''),
    method
  )
}

-- Build.
local elf = nil
local cargo <close> = os.spawn(
  {'cargo', stdout = 'pipe', env = env},
  'build',
  '-r',
  '--target', 'x86_64-unknown-none',
  '-Z', 'build-std=alloc,core',
  '--message-format', 'json-render-diagnostics'
)

while true do
  -- Read message.
  local msg = cargo.stdout:read()

  if not msg then
    break
  end

  msg = json.parse(msg)

  -- Check message type.
  local reason = msg.reason

  if reason == 'build-finished' then
    if msg.success then
      break
    else
      exit(1)
    end
  elseif reason == 'compiler-artifact' then
    if msg.target.name == 'kernel-dumper' then
      elf = msg.executable
    end
  end
end

-- Create payload.
os.run('rustup', 'run', 'nightly', 'objcopy', '-O', 'binary', elf, 'kernel-dumper.bin')
