-- Get method.
local fw = args.fw
local method = nil

if fw == '11.00' then
  method = args.dm or 'direct'
elseif fw == '11.52' then
  method = 'syscall'
end

-- Build environment variables.
local env = {
  RUSTFLAGS = string.format(
    '--cfg fw="%s" --cfg method="%s" -Z unstable-options -C panic=immediate-abort',
    fw:gsub('%.', ''),
    method
  )
}

-- Build arguments.
local spawn = {
  'build',
  '-r',
  '--target', 'x86_64-unknown-none',
  '-Z', 'build-std=alloc,core',
  '--message-format', 'json-render-diagnostics'
}

if args['patch-copyin'] then
  spawn[#spawn + 1] = '-F'
  spawn[#spawn + 1] = 'patch-copyin'
end

-- Build.
local elf = nil
local cargo <close> = os.spawn({'cargo', stdout = 'pipe', env = env}, table.unpack(spawn))

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
